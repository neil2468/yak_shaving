use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    str::{self, FromStr},
    thread,
};
use tracing::{info, Level};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "server")]
    server_ip: String,

    #[arg(long = "port1", default_value_t = 4000)]
    port1: u16,

    #[arg(long = "port2", default_value_t = 4001)]
    port2: u16,

    #[arg(long = "port3", default_value_t = 4002)]
    port3: u16,
}

fn main() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    info!("Args: {:?}", args);

    for port in [args.port1, args.port2, args.port3] {
        let local_addr = SocketAddr::from(([0, 0, 0, 0], 0));
        let socket = UdpSocket::bind(local_addr)?;
        let local_addr = socket.local_addr()?;

        let tmp = local_addr.to_string();
        let buf = tmp.as_bytes();

        socket.send_to(
            buf,
            SocketAddr::from((IpAddr::from_str(&args.server_ip)?, port)),
        )?;
    }

    Ok(())
}
