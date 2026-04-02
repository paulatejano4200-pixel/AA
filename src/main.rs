mod core;
mod miner;
mod node;

use clap::{Parser, Subcommand};
use crate::core::chain::genesis_block;
use crate::core::types::ChainParams;

#[derive(Parser)]
#[command(name = "aurum-cli")]
struct Cli { #[command(subcommand)] command: Commands }

#[derive(Subcommand)]
enum Commands { Init, ShowGenesis }

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Commands::Init => {
            let p = ChainParams::testnet();
            println!("Aurum testnet initialized | chain_id={} | reward={} grains", p.chain_id, p.initial_reward);
        }
        Commands::ShowGenesis => {
            let p = ChainParams::testnet();
            let g = genesis_block(&p);
            println!("genesis_height={} genesis_hash={}", g.header.height, hex::encode(g.hash()));
        }
    }
}
