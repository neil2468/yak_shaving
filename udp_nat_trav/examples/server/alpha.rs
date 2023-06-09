use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use tokio::{net::UdpSocket, task::JoinSet, time::sleep};
use tracing::{info, trace};

const PORTS: std::ops::RangeInclusive<u16> = 4000..=4009;
const THRESHOLD_PERCENT: usize = 80;

pub struct PeerData {
    rx_events: Vec<(SocketAddr, Instant)>,
}

impl PeerData {
    fn new() -> Self {
        Self {
            rx_events: Vec::new(),
        }
    }

    fn with_event(addr: SocketAddr) -> Self {
        let mut s = Self::new();
        s.record_rx_event(addr);
        s
    }

    fn record_rx_event(&mut self, addr: SocketAddr) {
        self.rx_events.push((addr, Instant::now()));
    }

    pub fn most_recent(&self) -> Option<Instant> {
        self.rx_events
            .iter()
            .map(|(_, instant)| instant)
            .max()
            .copied()
    }

    pub fn test_count(&self) -> usize {
        self.rx_events.len()
    }

    pub fn test_complete(&self) -> bool {
        let threshold = PORTS.len() * THRESHOLD_PERCENT / 100;
        self.rx_events.len() >= threshold
    }

    pub fn rx_events(&self) -> impl Iterator<Item = &SocketAddr> {
        self.rx_events.iter().map(|(addr, _)| addr)
    }

    pub fn conclusion(&self) -> Option<AlphaResult> {
        self.analysis()
            .max_by_key(|x| x.1)
            .and_then(|(res, _)| Some(res))
    }

    pub fn analysis(&self) -> impl Iterator<Item = (AlphaResult, usize)> {
        // (alpha_result, confidence 0..100)
        let mut results: Vec<(AlphaResult, usize)> = Vec::new(); //
        results.push((AlphaResult::Unknown, THRESHOLD_PERCENT));

        if self.test_complete() {
            // Prep work
            let test_count = self.test_count();
            let mut map: HashMap<SocketAddr, usize> = HashMap::new();
            for (addr, _) in &self.rx_events {
                map.entry(*addr)
                    .and_modify(|count| *count += 1)
                    .or_insert(1);
            }

            // Was the NAT source IP and port mostly the same
            for (addr, count) in &map {
                results.push((
                    AlphaResult::SrcIpPortConstant(addr.ip(), addr.port()),
                    usize::checked_div(count * 100, test_count).unwrap_or(0),
                ))
            }

            // Was the NAT source IP and port mostly different
            let count = map.iter().filter(|(_, &count)| count == 1).count();

            results.push((
                AlphaResult::SrcIpPortInconstant,
                usize::checked_div(count * 100, test_count).unwrap_or(0),
            ));
        }

        results.into_iter()
    }
}

#[derive(Debug)]
pub enum AlphaResult {
    Unknown,
    SrcIpPortInconstant,
    SrcIpPortConstant(IpAddr, u16),
}

pub struct AlphaManager {
    data: Arc<DashMap<String, PeerData>>,
}

impl AlphaManager {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn data(&self) -> Arc<DashMap<String, PeerData>> {
        self.data.clone()
    }

    pub fn spawn_tasks(&self, join_set: &mut JoinSet<Result<(), anyhow::Error>>) {
        for port in PORTS {
            let data_clone = self.data.clone();
            join_set.spawn(Self::port_listen_task(port, data_clone));
        }

        let data_clone = self.data.clone();
        join_set.spawn(Self::caretaker_task(data_clone));
    }

    async fn caretaker_task(data: Arc<DashMap<String, PeerData>>) -> anyhow::Result<()> {
        loop {
            sleep(Duration::from_millis(1000)).await; // TODO: Magic number

            // Delete expired data
            data.retain(|_, peer_data| {
                match peer_data.most_recent() {
                    Some(instant) => instant.elapsed() < Duration::from_millis(3000), // TODO: magic number
                    None => true,
                }
            })
        }
    }

    async fn port_listen_task(
        port: u16,
        data: Arc<DashMap<String, PeerData>>,
    ) -> anyhow::Result<()> {
        let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;
        let mut buf = [0; 1024]; // TODO: Is this large enough? Use a vec?

        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            let id = str::from_utf8(&buf[..len])?; // TODO: should not allow a utf8 issue stop the loop
            trace!("Rx on {}: {}", port, id);

            // Record event
            data.entry(String::from(id))
                .and_modify(|peer_data| peer_data.record_rx_event(addr))
                .or_insert(PeerData::with_event(addr));
        }
    }
}
