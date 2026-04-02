mod core;
mod miner;
mod node;

use std::sync::Arc;

use clap::{Parser, Subcommand};
use ed25519_dalek::SigningKey;
use parking_lot::RwLock;
use rand::rngs::OsRng;

use crate::core::chain::Blockchain;
use crate::core::tx::{Transaction, TxInput, TxOutput};
use crate::miner::{mine_one, miner_address_from_pubkey};
use crate::node::mempool::Mempool;
use crate::node::p2p::{run_p2p_listener, send_message, WireMsg};
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
    ShowUtxo { #[arg(long, default_value = "./data")] data_dir: String, #[arg(long, default_value_t = 20)] limit: usize },
    MempoolSelfTx { #[arg(long, default_value = "./data")] data_dir: String },
    P2pListen { #[arg(long, default_value = "127.0.0.1:4040")] bind: String },
    P2pSendStatus {
        #[arg(long, default_value = "./data")] data_dir: String,
        #[arg(long, default_value = "127.0.0.1:4040")] to: String,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init { data_dir } => init_chain(&data_dir)?,
        Commands::Mine { data_dir, threads } => mine_cmd(&data_dir, threads).await?,
        Commands::ShowTip { data_dir } => show_tip(&data_dir)?,
        Commands::ShowUtxo { data_dir, limit } => show_utxo(&data_dir, limit)?,
        Commands::MempoolSelfTx { data_dir } => mempool_self_tx(&data_dir)?,
        Commands::P2pListen { bind } => run_p2p_listener(&bind).await?,
        Commands::P2pSendStatus { data_dir, to } => p2p_send_status(&data_dir, &to).await?,
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

    let params = crate::core::types::ChainParams::testnet();
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

fn show_utxo(data_dir: &str, limit: usize) -> anyhow::Result<()> {
    let storage = Storage::open(&format!("{data_dir}/chain.db"))?;
    let utxos = storage.list_utxos(limit)?;
    println!("utxo_count_sample={}", utxos.len());
    for (i, ((txid, vout), e)) in utxos.iter().enumerate() {
        println!(
            "#{i} txid={} vout={} value={}",
            hex::encode(txid),
            vout,
            e.value
        );
    }
    Ok(())
}

fn mempool_self_tx(data_dir: &str) -> anyhow::Result<()> {
    let storage = Storage::open(&format!("{data_dir}/chain.db"))?;
    let chain = storage.load_chain()?.ok_or_else(|| anyhow::anyhow!("run init first"))?;

    let Some(((txid, vout), entry)) = chain.utxo.0.iter().next().map(|(k,v)| (*k, v.clone())) else {
        anyhow::bail!("no utxo available");
    };

    let sk = SigningKey::generate(&mut OsRng);

    let mut tx = Transaction {
        version: 1,
        locktime: 0,
        inputs: vec![TxInput {
            prev_txid: txid,
            prev_vout: vout,
            pubkey: vec![],
            sig: vec![],
        }],
        outputs: vec![TxOutput {
            value: entry.value.saturating_sub(1000),
            pubkey_hash: miner_address_from_pubkey(&sk.verifying_key().to_bytes()),
        }],
    };
    tx.sign_input(0, &sk);

    let mut mp = Mempool::new(10_000);
    mp.add_tx(tx.clone(), &chain.utxo)?;
    println!("mempool accepted tx: txid={}", hex::encode(tx.txid()));
    Ok(())
}

async fn p2p_send_status(data_dir: &str, to: &str) -> anyhow::Result<()> {
    let storage = Storage::open(&format!("{data_dir}/chain.db"))?;
    let chain = storage.load_chain()?.ok_or_else(|| anyhow::anyhow!("run init first"))?;
    let tip = chain.blocks.last().unwrap();

    let msg = WireMsg::Status {
        height: tip.header.height,
        tip_hash_hex: hex::encode(tip.hash()),
    };
    send_message(to, &msg).await?;
    println!("status sent to {}", to);
    Ok(())
}
