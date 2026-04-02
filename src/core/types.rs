use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};

pub type Hash256 = [u8; 32];
pub type Amount = u64;
pub const GRAINS_PER_AUR: Amount = 100_000_000;

pub fn target_from_hex(hex_str: &str) -> BigUint {
    BigUint::from_str_radix(hex_str, 16).expect("invalid target hex")
}

pub fn hash_to_big(hash: &Hash256) -> BigUint {
    BigUint::from_bytes_be(hash)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParams {
    pub chain_id: u32,
    pub max_block_size: usize,
    pub target_block_time_secs: u64,
    pub initial_target_hex: String,
    pub halving_interval: u64,
    pub initial_reward: Amount,
    pub max_supply: Amount,
}

impl ChainParams {
    pub fn testnet() -> Self {
        Self {
            chain_id: 0x41555254,
            max_block_size: 1_000_000,
            target_block_time_secs: 30,
            initial_target_hex: "00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
            halving_interval: 210_000,
            initial_reward: 1_666_666_667,
            max_supply: 7_000_000 * GRAINS_PER_AUR,
        }
    }
}
