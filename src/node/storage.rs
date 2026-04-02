use anyhow::Context;
use rocksdb::{Options, DB};

use crate::core::chain::Blockchain;

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
}
