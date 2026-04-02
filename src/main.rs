mod core;
mod miner;
mod node;

use std::sync::Arc;

use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use parking_lot::RwLock;
use rand::rngs::OsRng;

use crate::core::chain::Blockchain;
use crate::core::types::ChainParams;
use crate::miner::{mine_one, miner_address_from_pubkey};
use crate::node::mempool::Mempool;
use crate::node::storage::Storage;

#[derive(Parser)]
#[command(name = "aurum-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init { #[arg(long, default_value = "./data")] data_dir: String },
    Mine { #[arg(long, default_value = "./data")] data_dir: String, #[arg(long, default_value_t = 4)] threads: usize },
    ShowTip { #[arg(long, default_value = "./data")] data_dir: String },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { data_dir } => init_chain(&data_dir)?,
        Commands::Mine { data_dir, threads } => mine_cmd(&data_dir, threads).await?,
        Commands::ShowTip { data_dir } => show_tip(&data_dir)?,
    }

    Ok(())
}

fn init_chain(data_dir: &str) -> anyhow::Result<()> {
    std::fs::create_dir_all(data_dir)?;
    let storage = Storage::open(&format!("{data_dir}/chain.db"))?;

    if storage.load_chain()?.is_some() {
        println!("chain already initialized");
        return Ok(());
    }

    let params = ChainParams::testnet();
    let chain = Blockchain::new(params)?;
    storage.save_chain(&chain)?;

    println!("testnet initialized");
    Ok(())
}

async fn mine_cmd(data_dir: &str, threads: usize) -> anyhow::Result<()> {
    let storage = Storage::open(&format!("{data_dir}/chain.db"))?;
    let mut chain = storage.load_chain()?.ok_or_else(|| anyhow::anyhow!("run init first"))?;

    let sk = SigningKey::generate(&mut OsRng);
    let pkh = miner_address_from_pubkey(&sk.verifying_key().to_bytes());

    let chain_arc = Arc::new(RwLock::new(chain.clone()));
    let mempool = Arc::new(RwLock::new(Mempool::new(20_000)));

    if let Some(block) = mine_one(chain_arc, mempool, pkh, threads) {
        chain.apply_block(block.clone())?;
        storage.save_chain(&chain)?;

        println!(
            "mined: height={} hash={} coinbase_txid={} reward={}",
            block.header.height,
            hex::encode(block.hash()),
            hex::encode(block.body.transactions[0].txid()),
            block.body.transactions[0].outputs[0].value
        );
    } else {
        println!("mining interrupted");
    }

    Ok(())
}

fn show_tip(data_dir: &str) -> anyhow::Result<()> {
    let storage = Storage::open(&format!("{data_dir}/chain.db"))?;
    let chain = storage.load_chain()?.ok_or_else(|| anyhow::anyhow!("run init first"))?;
    let tip = chain.blocks.last().unwrap();

    println!(
        "tip: height={} hash={} txs={}",
        tip.header.height,
        hex::encode(tip.hash()),
        tip.body.transactions.len()
    );
    Ok(())
}
