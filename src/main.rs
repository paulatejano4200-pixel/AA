mod core;
mod miner;
mod node;

use clap::{Parser, Subcommand};

use crate::core::chain::genesis_block;
use crate::core::types::ChainParams;

#[derive(Parser)]
#[command(name = "aurum-cli")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init,
    ShowGenesis,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => {
            let params = ChainParams::testnet();
            println!(
                "Aurum testnet initialized | chain_id={} | reward={} grains",
                params.chain_id, params.initial_reward
            );
        }
        Commands::ShowGenesis => {
            let params = ChainParams::testnet();
            let g = genesis_block(&params);
            println!(
                "genesis_height={} genesis_hash={}",
                g.header.height,
                hex::encode(g.hash())
            );
        }
    }
}
