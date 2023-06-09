//! Prototype PEer (version 1) for UDP NAT Hole Punching

mod shared;

use clap::Parser;
use tokio::net::UdpSocket;
use tracing::info;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    id: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::setup_tracing()?;
    info!("Started");

    let args = Args::parse();
    info!("Args: {:?}", args);

    let socket = UdpSocket::bind(("0.0.0.0", 0)).await?;
    let buf = args.id.as_bytes();
    for port in shared::PORT_BASE..(shared::PORT_BASE + shared::PORT_COUNT) {
        let addr = ("127.0.0.1", port);
        socket.send_to(buf, addr).await?;
    }

    info!("Finished");
    Ok(())
}
