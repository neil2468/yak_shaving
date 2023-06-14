//! Prototype Rendezvous Server (version 1) for UDP NAT Hole Punching

mod server;
mod shared;

use crate::server::{AlphaManager, BetaManager};
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use tokio::{net::UdpSocket, task::JoinSet, time::sleep};
use tracing::{info, trace, warn};

// TODO: Characterize if we expect NAT to use a different source IP or port with
// another peer than it did with us.

// TODO: acknowledgements of received packets, and timeouts for peer and server.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::setup_tracing()?;
    info!("Started");

    // Spawn tasks for tests
    let alpha_manager = Arc::new(AlphaManager::new());
    let mut join_set = JoinSet::new();
    alpha_manager.spawn_tasks(&mut join_set);
    let beta_manager = Arc::new(BetaManager::new());
    beta_manager.spawn_tasks(&mut join_set);

    // Monitor task
    join_set.spawn(monitor_task(alpha_manager.clone(), beta_manager.clone()));

    // API task
    join_set.spawn(api_task(
        shared::API_PORT,
        alpha_manager.clone(),
        beta_manager.clone(),
    ));

    // Wait on tasks
    while let Some(res) = join_set.join_next().await {
        let _ = res?; // TODO: this does not seem right
    }

    info!("Finished");
    Ok(())
}

async fn api_task(
    port: u16,
    alpha_manager: Arc<AlphaManager>,
    beta_manager: Arc<BetaManager>,
) -> anyhow::Result<()> {
    loop {
        let socket = UdpSocket::bind((Ipv4Addr::new(0, 0, 0, 0), port)).await?;
        let mut buf = [0; 1024]; // TODO: Is this large enough? Use a vec?

        loop {
            let (len, _) = socket.recv_from(&mut buf).await?;

            if let Ok(msg) = serde_json::from_slice::<shared::Message>(&buf[0..len]) {
                info!("Rx: {:?}", msg);
                match msg {
                    shared::Message::QueryReq(payload) => {
                        let mut parts = payload.split('#');
                        let id = parts.next().expect("unexpected error");
                        let peer_id = parts.next().expect("unexpected error");

                        let id_alpha_conclusion = alpha_manager
                            .data()
                            .get(id)
                            .and_then(|peer_data| peer_data.conclusion());

                        let id_beta_conclusion = beta_manager
                            .data()
                            .get(id)
                            .and_then(|peer_data| peer_data.conclusion());

                        let peer_id_alpha_conclusion = alpha_manager
                            .data()
                            .get(peer_id)
                            .and_then(|peer_data| peer_data.conclusion());

                        let peer_id_beta_conclusion = beta_manager
                            .data()
                            .get(peer_id)
                            .and_then(|peer_data| peer_data.conclusion());
                    }
                    _ => todo!(),
                }
            } else {
                warn!("Unable to parse message");
            }
        }
    }
}

async fn monitor_task(
    alpha_manager: Arc<AlphaManager>,
    beta_manager: Arc<BetaManager>,
) -> anyhow::Result<()> {
    loop {
        sleep(Duration::from_millis(1000)).await; // TODO: magic number

        if alpha_manager.data().is_empty() == false || beta_manager.data().is_empty() == false {
            let data = alpha_manager.data();
            for ref_multi in data.iter() {
                let id = ref_multi.key();
                let peer_data = ref_multi.value();
                if let Some(instant) = peer_data.most_recent() {
                    if instant.elapsed() <= Duration::from_millis(1000) {
                        info!("Alpha...");
                        info!("  id: {}", id);
                        info!("    elapsed: {:?}", instant.elapsed());
                        info!("    analysis...");
                        peer_data.analysis().for_each(|x| info!("      {:?}", x));
                        trace!("    rx_events...");
                        peer_data.rx_events().for_each(|x| trace!("      {:?}", x));
                    }
                }
            }

            let data = beta_manager.data();
            for ref_multi in data.iter() {
                let id = ref_multi.key();
                let peer_data = ref_multi.value();
                if let Some(instant) = peer_data.most_recent() {
                    if instant.elapsed() <= Duration::from_millis(1000) {
                        info!("Beta...");
                        info!("  id: {}", id);
                        info!("    elapsed: {:?}", instant.elapsed());
                        info!("    analysis...");
                        peer_data.analysis().for_each(|x| info!("      {:?}", x));
                        trace!("    rx_events...");
                        peer_data.rx_events().for_each(|x| trace!("      {:?}", x));
                    }
                }
            }
        }
    }
}
