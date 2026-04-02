use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::core::types::{Amount, Hash256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub prev_txid: Hash256,
    pub prev_vout: u32,
    pub script_sig: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: Amount,
    pub pubkey_hash: Hash256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u16,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
    pub locktime: u32,
}

impl Transaction {
    pub fn txid(&self) -> Hash256 {
        let enc = bincode::serialize(self).expect("serialize tx");
        Sha256::digest(enc).into()
    }

    pub fn coinbase(height: u64, reward: Amount, message: &[u8]) -> Self {
        let mut marker = [0u8; 32];
        marker[..8].copy_from_slice(&height.to_le_bytes());

        Self {
            version: 1,
            inputs: vec![TxInput {
                prev_txid: marker,
                prev_vout: u32::MAX,
                script_sig: message.to_vec(),
            }],
            outputs: vec![TxOutput {
                value: reward,
                pubkey_hash: [0u8; 32],
            }],
            locktime: 0,
        }
    }
}
