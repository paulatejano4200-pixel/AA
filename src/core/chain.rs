use std::collections::{BTreeMap, HashSet};

use anyhow::{bail, ensure};
use serde::{Deserialize, Serialize};

use crate::core::block::{merkle_root, Block, BlockBody, BlockHeader, GENESIS_MESSAGE};
use crate::core::pow::meets_target;
use crate::core::tx::Transaction;
use crate::core::types::{target_from_hex, Amount, ChainParams, Hash256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoEntry {
    pub value: Amount,
    pub pubkey_hash: [u8; 32],
}

pub type OutPoint = (Hash256, u32);

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct UtxoSet(pub BTreeMap<OutPoint, UtxoEntry>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Blockchain {
    pub params: ChainParams,
    pub blocks: Vec<Block>,
    pub utxo: UtxoSet,
    pub money_created: Amount,
}

impl Blockchain {
    pub fn new(params: ChainParams) -> anyhow::Result<Self> {
        let mut chain = Self {
            params: params.clone(),
            blocks: vec![],
            utxo: UtxoSet::default(),
            money_created: 0,
        };
        let genesis = genesis_block(&params);
        chain.apply_block(genesis)?;
        Ok(chain)
    }

    pub fn tip_hash(&self) -> Hash256 {
        self.blocks.last().map(|b| b.hash()).unwrap_or([0u8; 32])
    }

    pub fn height(&self) -> u64 {
        self.blocks.last().map(|b| b.header.height).unwrap_or(0)
    }

    pub fn block_reward(&self, height: u64) -> Amount {
        let halvings = height / self.params.halving_interval;
        if halvings >= 64 {
            return 0;
        }
        self.params.initial_reward >> halvings
    }

    pub fn expected_target(&self) -> [u8; 32] {
        let big = target_from_hex(&self.params.initial_target_hex);
        let mut out = [0u8; 32];
        let b = big.to_bytes_be();
        out[32 - b.len()..].copy_from_slice(&b);
        out
    }

    pub fn validate_block(&self, block: &Block) -> anyhow::Result<()> {
        ensure!(!block.body.transactions.is_empty(), "empty block");
        ensure!(block.header.height == self.height() + 1, "invalid height");
        ensure!(block.header.previous_hash == self.tip_hash(), "prev hash mismatch");
        ensure!(
            block.header.merkle_root == merkle_root(&block.body.transactions),
            "bad merkle root"
        );
        ensure!(block.header.target == self.expected_target(), "unexpected target");
        ensure!(meets_target(&block.header.hash(), &block.header.target), "pow invalid");

        let txs = &block.body.transactions;
        let coinbase = &txs[0];
        ensure!(coinbase.inputs.len() == 1, "coinbase input count");
        ensure!(coinbase.inputs[0].prev_vout == u32::MAX, "coinbase marker");

        let mut spent: HashSet<OutPoint> = HashSet::new();
        let mut total_fees: i128 = 0;

        for tx in txs.iter().skip(1) {
            let mut in_sum: i128 = 0;
            let out_sum: i128 = tx.outputs.iter().map(|o| o.value as i128).sum();

            for (i, input) in tx.inputs.iter().enumerate() {
                let op = (input.prev_txid, input.prev_vout);
                ensure!(!spent.contains(&op), "double spend in block");
                let Some(utxo) = self.utxo.0.get(&op) else { bail!("missing utxo input") };
                ensure!(tx.verify_input_signature(i), "invalid signature");
                in_sum += utxo.value as i128;
                spent.insert(op);
            }

            ensure!(in_sum >= out_sum, "tx creates value");
            total_fees += in_sum - out_sum;
        }

        let allowed = self.block_reward(block.header.height) as i128 + total_fees;
        let cb_out: i128 = coinbase.outputs.iter().map(|o| o.value as i128).sum();
        ensure!(cb_out <= allowed, "coinbase exceeds subsidy+fees");

        Ok(())
    }

    pub fn apply_block(&mut self, block: Block) -> anyhow::Result<()> {
        if !self.blocks.is_empty() {
            self.validate_block(&block)?;
        }

        for tx in &block.body.transactions {
            let is_coinbase = tx.inputs.len() == 1 && tx.inputs[0].prev_vout == u32::MAX;

            if !is_coinbase {
                for i in &tx.inputs {
                    self.utxo.0.remove(&(i.prev_txid, i.prev_vout));
                }
            }

            let txid = tx.txid();
            for (vout, out) in tx.outputs.iter().enumerate() {
                self.utxo.0.insert(
                    (txid, vout as u32),
                    UtxoEntry {
                        value: out.value,
                        pubkey_hash: out.pubkey_hash,
                    },
                );
            }
        }

        let minted: Amount = block.body.transactions[0].outputs.iter().map(|o| o.value).sum();
        self.money_created = self.money_created.saturating_add(minted);
        ensure!(self.money_created <= self.params.max_supply, "max supply exceeded");

        self.blocks.push(block);
        Ok(())
    }

    pub fn build_candidate_block(
        &self,
        txs: Vec<Transaction>,
        miner_pkh: [u8; 32],
        timestamp: u64,
    ) -> Block {
        let height = self.height() + 1;

        let mut fees: i128 = 0;
        for tx in &txs {
            let in_sum: i128 = tx
                .inputs
                .iter()
                .filter_map(|i| self.utxo.0.get(&(i.prev_txid, i.prev_vout)).map(|u| u.value as i128))
                .sum();

            let out_sum: i128 = tx.outputs.iter().map(|o| o.value as i128).sum();
            fees += (in_sum - out_sum).max(0);
        }

        let coinbase = Transaction::coinbase(
            height,
            self.block_reward(height) + fees.max(0) as u64,
            miner_pkh,
            b"aurum-coinbase".to_vec(),
        );

        let mut all = vec![coinbase];
        all.extend(txs);

        Block {
            header: BlockHeader {
                version: 1,
                height,
                previous_hash: self.tip_hash(),
                merkle_root: merkle_root(&all),
                timestamp,
                target: self.expected_target(),
                nonce: 0,
            },
            body: BlockBody { transactions: all },
        }
    }
}

pub fn genesis_block(params: &ChainParams) -> Block {
    let coinbase = Transaction::coinbase(0, params.initial_reward, [0u8; 32], GENESIS_MESSAGE.as_bytes().to_vec());

    Block {
        header: BlockHeader {
            version: 1,
            height: 0,
            previous_hash: [0u8; 32],
            merkle_root: merkle_root(&[coinbase.clone()]),
            timestamp: 1_711_929_600,
            target: {
                let mut t = [0u8; 32];
                let b = target_from_hex(&params.initial_target_hex).to_bytes_be();
                t[32 - b.len()..].copy_from_slice(&b);
                t
            },
            nonce: 0,
        },
        body: BlockBody {
            transactions: vec![coinbase],
        },
    }
}
