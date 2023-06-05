//! Prototype Rendezvous Server (version 1) for UDP NAT Hole Punching

use std::{
    collections::{HashMap, HashSet},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str,
};

use tokio::net::UdpSocket;
use tracing::info;

mod shared;

// TODO: Characterize if we expect NAT to use a different source IP or port with
// another peer than it did with us.

struct PeerStats {
    addrs: Vec<SocketAddr>,
}

impl PeerStats {
    fn new() -> Self {
        // TODO: will have id and other fields in future
        Self { addrs: Vec::new() }
    }

    fn with_addr(addr: SocketAddr) -> Self {
        let mut x = Self::new();
        x.insert_rx_addr(addr);
        x
    }

    fn insert_rx_addr(&mut self, addr: SocketAddr) {
        self.addrs.push(addr);
    }

    fn ip_stats(&self) -> HashMap<IpAddr, usize> {
        let mut map = HashMap::new();
        for addr in &self.addrs {
            map.entry(addr.ip())
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        map
    }

    fn port_stats(&self, ip: &IpAddr) -> HashMap<u16, usize> {
        let mut map = HashMap::new();
        for addr in &self.addrs {
            if ip == &addr.ip() {
                map.entry(addr.port())
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }
        }
        map
    }
}

#[derive(Debug)]
struct IpStats {
    times_seen: usize,
    port_stats: HashMap<u16, PortStats>,
}

#[derive(Debug)]
struct PortStats {
    times_seen: usize,
}

#[derive(Debug, Default)]
struct Peer {
    id: String, // TODO: currently not used
    rx_stats: HashMap<IpAddr, IpStats>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::setup_tracing()?;
    info!("Started");

    let mut map_peer_stats: HashMap<String, PeerStats> = HashMap::new();

    let PORTS = 4000..=4009;
    let mut sockets = Vec::new();
    for port in PORTS {
        let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;
        sockets.push(socket);
    }

    let mut buf = [0; 1024];
    for socket in &sockets {
        let (len, addr) = socket.recv_from(&mut buf).await?;

        let id = str::from_utf8(&buf[..len])?;

        map_peer_stats
            .entry(String::from(id))
            .and_modify(|stats| stats.insert_rx_addr(addr))
            .or_insert(PeerStats::with_addr(addr));
    }

    for (id, peer_stats) in map_peer_stats {
        info!("id: {}", id);
        for (ip, count) in peer_stats.ip_stats() {
            info!("  ip: {}, {}", ip, count);
            for (port, count) in peer_stats.port_stats(&ip) {
                info!("    port: {}, {}", port, count);
            }
        }
    }

    info!("Finished");
    Ok(())
}
