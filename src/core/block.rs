use crate::core::pow::arh256_hash_v2;
use crate::core::tx::Transaction;
use crate::core::types::Hash256;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const GENESIS_MESSAGE: &str = "Aurum: Sovereignty restored to the common PC. April 2026.";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub version: u16,
    pub height: u64,
    pub previous_hash: Hash256,
    pub merkle_root: Hash256,
    pub timestamp: u64,
    pub target: [u8; 32],
    pub nonce: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBody {
    pub transactions: Vec<Transaction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub body: BlockBody,
}

impl BlockHeader {
    pub fn hash(&self) -> Hash256 {
        arh256_hash_v2(&bincode::serialize(self).expect("serialize header"))
    }
}
impl Block { pub fn hash(&self) -> Hash256 { self.header.hash() } }

pub fn merkle_root(txs: &[Transaction]) -> Hash256 {
    if txs.is_empty() { return [0u8; 32]; }
    let mut layer: Vec<Hash256> = txs.iter().map(|tx| tx.txid()).collect();
    while layer.len() > 1 {
        if layer.len() % 2 == 1 { layer.push(*layer.last().unwrap()); }
        let mut next = Vec::with_capacity(layer.len() / 2);
        for pair in layer.chunks(2) {
            let mut h = Sha256::new();
            h.update(pair[0]);
            h.update(pair[1]);
            next.push(h.finalize().into());
        }
        layer = next;
    }
    layer[0]
}
