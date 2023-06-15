//! Prototype of peer who's behind a domestic router and which wants to connect
//! to alice.

use clap::Parser;
use std::{
    collections::HashSet,
    net::{IpAddr, SocketAddr},
};
use tokio::net::UdpSocket;
use tracing::{info, trace, Level};

const ID: &str = "bob";
const PEER_ID: &str = "alice";
const JUST_IN_CASE: usize = 3;

// TODO: Constants to rename
const ALPHA: usize = 220;
const BETA: usize = 1024;

#[derive(Parser, Debug)]
#[command()]
struct Args {
    #[arg(long)]
    peer_ip: IpAddr,
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

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    info!("Local socket addr = {:?}", socket.local_addr()?);

    let mut ports: HashSet<u16> = HashSet::with_capacity(ALPHA); // TODO: magic number
    while ports.len() < ALPHA {
        ports.insert(rand::random());
    }
    info!("Generated {} random port numbers", ports.len());

    punch_local_holes(&socket, &args.peer_ip, &ports).await?;

    handle_pings(&socket).await?;

    todo!()
}

async fn punch_local_holes(
    socket: &UdpSocket,
    ip: &IpAddr,
    ports: &HashSet<u16>,
) -> anyhow::Result<()> {
    let old_ttl = socket.ttl()?;
    socket.set_ttl(3)?;

    for port in ports.iter() {
        let addr = SocketAddr::new(*ip, *port);
        let buf = String::from("ping");
        let buf = buf.as_bytes();
        for _ in 0..JUST_IN_CASE {
            socket.send_to(buf, addr).await?;
        }
        trace!("Punched local hole to {:?} (ttl={})", addr, socket.ttl()?);
    }

    info!(
        "Punched {} local holes (ttl={})",
        ports.len(),
        socket.ttl()?
    );

    socket.set_ttl(old_ttl)?;

    Ok(())
}

async fn handle_pings(socket: &UdpSocket) -> anyhow::Result<()> {
    let mut buf = [0; BETA];

    info!("Will handle pings");
    loop {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        if let Ok(buf_str) = std::str::from_utf8(&buf[0..len]) {
            if buf_str.starts_with("ping") {
                info!("Rx ping from {:?}: {:?}", addr, buf_str);
                // TODO: reply to ping
            }
        }
    }

    Ok(())
}
