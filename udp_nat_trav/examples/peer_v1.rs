//! Prototype PEer (version 1) for UDP NAT Hole Punching

mod shared;

use std::time::Duration;

use clap::Parser;
use tokio::{net::UdpSocket, time::sleep};
use tracing::info;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    id: String,

    #[arg(long)]
    peer_id: String,

    #[arg(long)]
    server: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::setup_tracing()?;
    info!("Started");

    let args = Args::parse();
    info!("Args: {:?}", args);

    // Alpha tests
    let socket = UdpSocket::bind(("0.0.0.0", 0)).await?;
    let buf = args.id.as_bytes();
    for port in shared::ALPHA_PORT_BASE..(shared::ALPHA_PORT_BASE + shared::ALPHA_PORT_COUNT) {
        let addr = (args.server.clone(), port);
        socket.send_to(buf, addr).await?;
    }

    // Beta tests
    for seq_num in 0..shared::BETA_COUNT {
        let socket = UdpSocket::bind(("0.0.0.0", 0)).await?;

        let buf = format!(
            "{}#{}#{}",
            args.id,
            socket.local_addr().unwrap().port(),
            seq_num
        );
        let buf = buf.as_bytes();

        let addr = (args.server.clone(), shared::BETA_PORT);
        socket.send_to(buf, addr).await?;
    }

    // Query peer
    sleep(Duration::from_millis(1000)).await;
    let socket = UdpSocket::bind(("0.0.0.0", 0)).await?;
    let msg = shared::Message::QueryReq(format!("{}#{}", args.id, args.peer_id));
    let buf = serde_json::to_string(&msg)?;
    let buf = buf.as_bytes();
    let addr = (args.server.clone(), shared::API_PORT);
    socket.send_to(buf, addr).await?;

    info!("Finished");
    Ok(())
}
