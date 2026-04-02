use serde::{Deserialize, Serialize};

pub type Hash256 = [u8; 32];
pub type Amount = u64;

pub const GRAINS_PER_AUR: Amount = 100_000_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainParams {
    pub chain_id: u32,
    pub initial_reward: Amount,
    pub halving_interval: u64,
    pub max_supply: Amount,
    pub initial_target_hex: String,
}

impl ChainParams {
    pub fn testnet() -> Self {
        Self {
            chain_id: 0x41555254,
            initial_reward: 1_666_666_667,
            halving_interval: 210_000,
            max_supply: 7_000_000 * GRAINS_PER_AUR,
            initial_target_hex: "00000fffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".into(),
        }
    }
}
