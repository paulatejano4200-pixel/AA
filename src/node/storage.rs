use anyhow::Context;
use rocksdb::{Options, DB};

use crate::core::chain::{Blockchain, OutPoint, UtxoEntry};
use crate::core::types::Hash256;

pub struct Storage {
    db: DB,
}

impl Storage {
    pub fn open(path: &str) -> anyhow::Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        let db = DB::open(&opts, path)?;
        Ok(Self { db })
    }

    pub fn save_chain(&self, chain: &Blockchain) -> anyhow::Result<()> {
        let raw = bincode::serialize(chain)?;
        self.db.put(b"chain", raw)?;
        Ok(())
    }

    pub fn load_chain(&self) -> anyhow::Result<Option<Blockchain>> {
        let Some(raw) = self.db.get(b"chain")? else {
            return Ok(None);
        };
        let chain = bincode::deserialize(&raw).context("deserialize chain")?;
        Ok(Some(chain))
    }

    pub fn get_utxo(&self, txid: Hash256, vout: u32) -> anyhow::Result<Option<UtxoEntry>> {
        let Some(chain) = self.load_chain()? else { return Ok(None) };
        Ok(chain.utxo.0.get(&(txid, vout)).cloned())
    }

    pub fn list_utxos(&self, limit: usize) -> anyhow::Result<Vec<(OutPoint, UtxoEntry)>> {
        let Some(chain) = self.load_chain()? else { return Ok(vec![]) };
        Ok(chain
            .utxo
            .0
            .iter()
            .take(limit)
            .map(|(k, v)| (*k, v.clone()))
            .collect())
    }
}
