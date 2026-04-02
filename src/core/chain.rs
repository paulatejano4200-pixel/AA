use crate::core::block::{merkle_root, Block, BlockBody, BlockHeader, GENESIS_MESSAGE};
use crate::core::tx::Transaction;
use crate::core::types::ChainParams;

pub fn genesis_block(params: &ChainParams) -> Block {
    let coinbase = Transaction::coinbase(0, params.initial_reward, [0u8; 32], GENESIS_MESSAGE.as_bytes().to_vec());
    Block {
        header: BlockHeader {
            version: 1,
            height: 0,
            previous_hash: [0u8; 32],
            merkle_root: merkle_root(&[coinbase.clone()]),
            timestamp: 1_711_929_600,
            target: [0x0f; 32],
            nonce: 0,
        },
        body: BlockBody { transactions: vec![coinbase] },
    }
}
