use crate::core::types::{Amount, Hash256};
use ed25519_dalek::{Signer, Signature, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxInput {
    pub prev_txid: Hash256,
    pub prev_vout: u32,
    pub pubkey: Vec<u8>,
    pub sig: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TxOutput {
    pub value: Amount,
    pub pubkey_hash: [u8; 32],
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub version: u16,
    pub locktime: u32,
    pub inputs: Vec<TxInput>,
    pub outputs: Vec<TxOutput>,
}

impl Transaction {
    pub fn txid(&self) -> Hash256 {
        Sha256::digest(bincode::serialize(self).expect("serialize tx")).into()
    }

    pub fn sighash(&self) -> Hash256 {
        let mut clone = self.clone();
        for i in &mut clone.inputs {
            i.sig.clear();
        }
        Sha256::digest(bincode::serialize(&clone).expect("serialize sighash")).into()
    }

    pub fn coinbase(height: u64, reward: Amount, miner_pubkey_hash: [u8; 32], extra: Vec<u8>) -> Self {
        let mut seed = [0u8; 32];
        seed[..8].copy_from_slice(&height.to_le_bytes());
        Self {
            version: 1,
            locktime: 0,
            inputs: vec![TxInput { prev_txid: seed, prev_vout: u32::MAX, pubkey: extra, sig: vec![] }],
            outputs: vec![TxOutput { value: reward, pubkey_hash: miner_pubkey_hash }],
        }
    }

    pub fn sign_input(&mut self, i: usize, sk: &SigningKey) {
        let sig = sk.sign(&self.sighash());
        self.inputs[i].sig = sig.to_bytes().to_vec();
        self.inputs[i].pubkey = sk.verifying_key().to_bytes().to_vec();
    }

    pub fn verify_input_signature(&self, i: usize) -> bool {
        let Some(input) = self.inputs.get(i) else { return false; };
        let Ok(pk_bytes) = <[u8; 32]>::try_from(input.pubkey.as_slice()) else { return false; };
        let Ok(sig_bytes) = <[u8; 64]>::try_from(input.sig.as_slice()) else { return false; };
        let Ok(pk) = VerifyingKey::from_bytes(&pk_bytes) else { return false; };
        let sig = Signature::from_bytes(&sig_bytes);
        pk.verify(&self.sighash(), &sig).is_ok()
    }
}

pub fn pubkey_hash(pubkey: &[u8]) -> [u8; 32] {
    Sha256::digest(pubkey).into()
}
