use anyhow::Context;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{TcpListener, TcpStream};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WireMsg {
    Status { height: u64, tip_hash_hex: String },
    Tx { txid_hex: String, raw_hex: String },
    Block { hash_hex: String },
}

pub async fn run_p2p_listener(bind_addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(bind_addr).await?;
    println!("p2p listening on {}", bind_addr);

    loop {
        let (socket, peer) = listener.accept().await?;
        tokio::spawn(async move {
            if let Err(e) = handle_conn(socket).await {
                eprintln!("p2p peer {} error: {}", peer, e);
            }
        });
    }
}

async fn handle_conn(socket: TcpStream) -> anyhow::Result<()> {
    let mut reader = BufReader::new(socket);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }
        let msg: WireMsg = serde_json::from_str(line.trim()).context("decode wire msg")?;
        println!("p2p recv: {:?}", msg);
    }

    Ok(())
}

pub async fn send_message(addr: &str, msg: &WireMsg) -> anyhow::Result<()> {
    let mut socket = TcpStream::connect(addr).await?;
    let raw = serde_json::to_string(msg)?;
    socket.write_all(raw.as_bytes()).await?;
    socket.write_all(b"\n").await?;
    Ok(())
}
