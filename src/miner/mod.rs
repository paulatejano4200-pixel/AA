use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;

use parking_lot::RwLock;

use crate::core::block::Block;
use crate::core::chain::Blockchain;
use crate::core::pow::meets_target;
use crate::core::tx::pubkey_hash;
use crate::node::mempool::Mempool;

pub fn mine_one(
    chain: Arc<RwLock<Blockchain>>,
    mempool: Arc<RwLock<Mempool>>,
    miner_pkh: [u8; 32],
    threads: usize,
) -> Option<Block> {
    let timestamp = now();
    let txs = mempool.read().top_by_fee(2048);
    let candidate = chain.read().build_candidate_block(txs, miner_pkh, timestamp);
    let target = candidate.header.target;

    let done = Arc::new(AtomicBool::new(false));
    let result = Arc::new(RwLock::new(None));

    thread::scope(|scope| {
        for tid in 0..threads {
            let done = done.clone();
            let result = result.clone();
            let mut blk = candidate.clone();

            scope.spawn(move || {
                let mut nonce = tid as u64;
                while !done.load(Ordering::Relaxed) {
                    blk.header.nonce = nonce;
                    let h = blk.header.hash();

                    if meets_target(&h, &target) {
                        *result.write() = Some(blk.clone());
                        done.store(true, Ordering::Relaxed);
                        break;
                    }

                    nonce = nonce.wrapping_add(threads as u64);
                }
            });
        }
    });

    let mined = result.write().take();
    mined
}

pub fn miner_address_from_pubkey(pubkey: &[u8]) -> [u8; 32] {
    pubkey_hash(pubkey)
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
