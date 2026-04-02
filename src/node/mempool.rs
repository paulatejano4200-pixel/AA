use std::collections::{BTreeMap, HashSet};

use crate::core::chain::{OutPoint, UtxoSet};
use crate::core::tx::Transaction;

#[derive(Default)]
pub struct Mempool {
    by_fee: BTreeMap<u64, Vec<Transaction>>,
    spent: HashSet<OutPoint>,
    max_txs: usize,
}

impl Mempool {
    pub fn new(max_txs: usize) -> Self {
        Self { by_fee: BTreeMap::new(), spent: HashSet::new(), max_txs }
    }

    pub fn add_tx(&mut self, tx: Transaction, utxo: &UtxoSet) -> anyhow::Result<()> {
        if self.len() >= self.max_txs {
            anyhow::bail!("mempool full");
        }

        let mut in_sum = 0u64;
        let mut out_sum = 0u64;

        for i in &tx.inputs {
            let op = (i.prev_txid, i.prev_vout);
            if self.spent.contains(&op) {
                anyhow::bail!("double spend in mempool");
            }
            let Some(u) = utxo.0.get(&op) else { anyhow::bail!("input not found") };
            in_sum = in_sum.saturating_add(u.value);
        }

        for o in &tx.outputs {
            out_sum = out_sum.saturating_add(o.value);
        }

        if in_sum < out_sum {
            anyhow::bail!("invalid fee");
        }

        let fee = in_sum - out_sum;
        for i in &tx.inputs {
            self.spent.insert((i.prev_txid, i.prev_vout));
        }
        self.by_fee.entry(fee).or_default().push(tx);

        Ok(())
    }

    pub fn top_by_fee(&self, max: usize) -> Vec<Transaction> {
        self.by_fee
            .iter()
            .rev()
            .flat_map(|(_, v)| v.iter().cloned())
            .take(max)
            .collect()
    }

    pub fn len(&self) -> usize {
        self.by_fee.values().map(|v| v.len()).sum()
    }
}
