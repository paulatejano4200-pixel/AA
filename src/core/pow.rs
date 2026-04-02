use blake3::Hasher;
use sha2::{Digest, Sha256};
use crate::core::types::{hash_to_big, Hash256};

pub fn arh256_hash_v2(data: &[u8]) -> Hash256 {
    const MEM_SIZE: usize = 4 * 1024 * 1024;
    const WORDS: usize = MEM_SIZE / 32;

    let seed: [u8; 32] = Sha256::digest(data).into();
    let mut mem = vec![[0u8; 32]; WORDS];
    let mut state = seed;

    for slot in mem.iter_mut() {
        let mut h = Hasher::new();
        h.update(&state);
        state = *h.finalize().as_bytes();
        *slot = state;
    }

    let mut acc = state;
    for i in 0..(WORDS * 2) {
        let idx = (u64::from_le_bytes(acc[0..8].try_into().unwrap()) as usize + i) % WORDS;
        let idx2 = (u64::from_le_bytes(acc[8..16].try_into().unwrap()) as usize ^ idx) % WORDS;
        let mut h = Hasher::new();
        h.update(&acc);
        h.update(&mem[idx]);
        h.update(&mem[idx2]);
        acc = *h.finalize().as_bytes();
        mem[idx] = acc;
    }

    Sha256::digest(acc).into()
}

pub fn meets_target(hash: &Hash256, target: &[u8; 32]) -> bool {
    hash_to_big(hash) < hash_to_big(target)
}
