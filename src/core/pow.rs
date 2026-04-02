use blake3::Hasher;
use sha2::{Digest, Sha256};

use crate::core::types::Hash256;

pub fn arh256_hash_v2(data: &[u8]) -> Hash256 {
    const MEM_SIZE: usize = 4 * 1024 * 1024;
    const WORDS: usize = MEM_SIZE / 32;

    let seed: [u8; 32] = Sha256::digest(data).into();
    let mut mem = vec![[0u8; 32]; WORDS];

    let mut state = seed;
    for (i, slot) in mem.iter_mut().enumerate() {
        let mut h = Hasher::new();
        h.update(&state);
        h.update(&(i as u64).to_le_bytes());
        state = *h.finalize().as_bytes();
        *slot = state;
    }

    let mut acc = state;
    for i in 0..(WORDS * 2) {
        let a = u64::from_le_bytes(acc[0..8].try_into().unwrap()) as usize;
        let b = u64::from_le_bytes(acc[8..16].try_into().unwrap()) as usize;
        let i1 = (a ^ i) % WORDS;
        let i2 = (b.wrapping_add(i1 * 1315423911)) % WORDS;

        let mut h = Hasher::new();
        h.update(&acc);
        h.update(&mem[i1]);
        h.update(&mem[i2]);
        acc = *h.finalize().as_bytes();

        mem[i1] = acc;
    }

    Sha256::digest(acc).into()
}
