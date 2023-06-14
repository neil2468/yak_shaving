use std::{
    collections::HashMap,
    net::{Ipv4Addr, SocketAddr},
    str,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use tokio::{net::UdpSocket, task::JoinSet, time::sleep};
use tracing::{info, trace};

const PORT: u16 = 4010;
const EXPECTED_TEST_COUNT: usize = 10;
const THRESHOLD_PERCENT: usize = 80;

pub struct PeerData {
    rx_events: Vec<(SocketAddr, u16, u16, Instant)>,
}

impl PeerData {
    fn new() -> Self {
        Self {
            rx_events: Vec::new(),
        }
    }

    fn with_event(addr: SocketAddr, orig_port: u16, seq_num: u16) -> Self {
        let mut s = Self::new();
        s.record_rx_event(addr, orig_port, seq_num);
        s
    }

    fn record_rx_event(&mut self, addr: SocketAddr, orig_port: u16, seq_num: u16) {
        self.rx_events
            .push((addr, orig_port, seq_num, Instant::now()));
    }

    pub fn most_recent(&self) -> Option<Instant> {
        self.rx_events
            .iter()
            .map(|(_, _, _, instant)| instant)
            .max()
            .copied()
    }

    pub fn test_count(&self) -> usize {
        self.rx_events.len()
    }

    pub fn test_complete(&self) -> bool {
        // TODO: allow for missed UDP packets, but the way below is not the way
        // we should go, I think
        let threshold = EXPECTED_TEST_COUNT * THRESHOLD_PERCENT / 100;
        self.rx_events.len() >= threshold
    }

    pub fn rx_events(&self) -> impl Iterator<Item = (&SocketAddr, &u16, &u16)> {
        self.rx_events
            .iter()
            .map(|(addr, orig_port, seq_num, _)| (addr, orig_port, seq_num))
    }

    pub fn conclusion(&self) -> Option<BetaResult> {
        self.analysis()
            .max_by_key(|x| x.1)
            .and_then(|(res, _)| Some(res))
    }

    pub fn analysis(&self) -> impl Iterator<Item = (BetaResult, usize)> {
        // (alpha_result, confidence 0..100)
        let mut results: HashMap<BetaResult, usize> = HashMap::new();
        results.insert(BetaResult::Unknown, THRESHOLD_PERCENT);

        if self.test_complete() {
            // Prep work
            let test_count = self.test_count();

            // map: <nat_port - orig_port, count>
            let mut map: HashMap<i32, usize> = HashMap::new();
            for (addr, orig_port, _, _) in &self.rx_events {
                let diff = addr.port() as i32 - *orig_port as i32;
                map.entry(diff).and_modify(|count| *count += 1).or_insert(1);
            }

            // Did the NAT use the same port as its client?
            let confidence = if map.len() == 1 && map.keys().next().unwrap() == &0 {
                100
            } else {
                0
            };
            results.insert(BetaResult::SrcPortAsOrig, confidence);

            // TODO: does this description make sense?
            // Did the NAT use ports which were a constant difference from the client?
            let confidence = if map.len() == 1 && map.keys().next().unwrap() != &0 {
                100
            } else {
                0
            };
            results.insert(BetaResult::SrcPortConstantDiffToOrig, confidence);

            // Did the NAT use ports close to what its client used?
            let count: usize = map
                .iter()
                .filter(|(&k, _)| k != 0 && k.abs() <= 100)
                .map(|(_, v)| v)
                .sum(); // TODO: magic number
            let confidence = usize::checked_div(count * 100, test_count).unwrap_or(0);
            results.insert(BetaResult::SrcPortCloseToOrig, confidence);

            // Did the NAT use ports on a round robin basis?
            let mut tmp: Vec<_> = self
                .rx_events
                .iter()
                .map(|(addr, _, seq_num, _)| (addr, seq_num))
                .collect();
            tmp.sort_by_key(|x| x.1);
            let mut diffs = Vec::with_capacity(tmp.len() - 1);
            let mut iter = tmp.iter().map(|(&addr, _)| addr.port());
            if let Some(mut prev) = iter.next() {
                for next in iter {
                    diffs.push(next.wrapping_sub(prev));
                    prev = next;
                }
            }

            let count = diffs.iter().filter(|&d| d <= &5000).count(); // TODO: magic number
            let confidence = usize::checked_div(count * 100, diffs.len()).unwrap_or(0);
            results.insert(BetaResult::SrcPortRoundRobin, confidence);

            // Did the NAT use random ports?
            // let count = match map.len() {
            //     0 | 1 => 0,
            //     l => l,
            // };
            // let confidence = usize::checked_div(count * 100, test_count).unwrap_or(0);
            // results.insert(BetaResult::SrcPortRandom, confidence);
        }

        results.into_iter()
    }
}

// TODO: purpose of this result is to decide which port to expect NAT to use with
// other peer. Useful options are, an exact port, a narrow port range, a wide port
// range (for is random and we do what we can).

// TODO: Could also have 'narrow range, but expect to be not less than previously
// used (with wrapping)'? - NarrowRangeDataPointIsStart, NarrowRangeDataPointIsCenter.

#[derive(Debug, PartialEq, Eq, Hash)]
pub enum BetaResult {
    Unknown,
    SrcPortAsOrig,
    SrcPortConstantDiffToOrig, // TODO: same use as SrcPortAsOrig?
    SrcPortCloseToOrig, // TODO: Valid but if this and SrcPorAsOrig are 100% AsOrig takes priority
    SrcPortRoundRobin,  // TODO: AKA CloseToPrev
                        // SrcPortRoundRobinWithGaps,
                        // SrcPortNarrowRange,
                        // SrcPortWideRange,
                        // SrcPortRandom, // TODO: Is this unknown?
}

pub struct BetaManager {
    data: Arc<DashMap<String, PeerData>>,
}

impl BetaManager {
    pub fn new() -> Self {
        Self {
            data: Arc::new(DashMap::new()),
        }
    }

    pub fn data(&self) -> Arc<DashMap<String, PeerData>> {
        self.data.clone()
    }

    pub fn spawn_tasks(&self, join_set: &mut JoinSet<Result<(), anyhow::Error>>) {
        let data_clone = self.data.clone();
        join_set.spawn(Self::port_listen_task(PORT, data_clone));

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
            let payload = str::from_utf8(&buf[..len])?; // TODO: should not allow a utf8 issue stop the loop
            trace!("Rx on {}: {}", port, payload);

            // Parse payload
            let mut parts = payload.split('#');
            let id = parts.next().expect("unexpected error");
            let orig_port: u16 = parts
                .next()
                .expect("unexpected error")
                .parse()
                .expect("failed to parse payload");
            let seq_num: u16 = parts
                .next()
                .expect("unexpected error")
                .parse()
                .expect("failed to parse payload");

            // Record event
            data.entry(String::from(id))
                .and_modify(|peer_data| peer_data.record_rx_event(addr, orig_port, seq_num))
                .or_insert(PeerData::with_event(addr, orig_port, seq_num));
        }
    }
}
