//! Prototype of peer who's behind the 1p mobile CGNAT
//! and which wants to connect to peer bob.

use clap::Parser;
use std::{collections::HashSet, net::SocketAddr, time::Duration};
use tokio::{net::UdpSocket, time::sleep};
use tracing::{info, Level};

const ID: &str = "alice";
const PEER_ID: &str = "bob";

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    peer_addr: SocketAddr,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    info!("Started");

    let args = Args::parse();
    info!("Args: {:?}", args);

    let mut ports: HashSet<u16> = HashSet::with_capacity(u16::MAX as usize);
    let mut attempt = 0;

    while attempt < 300 {
        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        let port = socket.local_addr()?.port();
        if ports.insert(port) == false {
            info!("Avoiding port reuse <<<<<<<<<<<<<<<<<");
        } else {
            attempt += 1;
            info!("Attempt {}. Send ping from {}", attempt, port);

            // Loop to mitigate lost UDP packets
            for _ in 0..3 {
                socket
                    .send_to(format!("ping#{}", attempt).as_bytes(), args.peer_addr)
                    .await?;
            }
            sleep(Duration::from_millis(100)).await;
        }
        drop(socket);
    }
    Ok(())
}
