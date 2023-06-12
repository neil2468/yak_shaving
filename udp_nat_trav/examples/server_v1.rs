//! Prototype Rendezvous Server (version 1) for UDP NAT Hole Punching

mod server;
mod shared;

use crate::server::{AlphaManager, BetaManager};
use std::{sync::Arc, time::Duration};
use tokio::{task::JoinSet, time::sleep};
use tracing::info;

// TODO: Characterize if we expect NAT to use a different source IP or port with
// another peer than it did with us.

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

    // Wait on tasks
    while let Some(res) = join_set.join_next().await {
        let _ = res?; // TODO: this does not seem right
    }

    info!("Finished");
    Ok(())
}

async fn monitor_task(
    alpha_manager: Arc<AlphaManager>,
    beta_manager: Arc<BetaManager>,
) -> anyhow::Result<()> {
    loop {
        sleep(Duration::from_millis(1000)).await; // TODO: magic number

        info!("Alpha...");
        let data = alpha_manager.data();
        for ref_multi in data.iter() {
            let id = ref_multi.key();
            let peer_data = ref_multi.value();

            info!("  id: {}", id);

            if let Some(instant) = peer_data.most_recent() {
                info!("    elapsed: {:?}", instant.elapsed());
            }

            info!("    analysis...");
            for x in peer_data.analysis() {
                info!("      {:?}", x);
            }
            info!(
                "    conclusion: {:?}",
                peer_data.analysis().max_by_key(|x| x.1)
            );

            info!("    rx_events...");
            for event in peer_data.rx_events() {
                info!("      {:?}", event);
            }
        }

        info!("Beta...");
        let data = beta_manager.data();
        for ref_multi in data.iter() {
            let id = ref_multi.key();
            let peer_data = ref_multi.value();

            info!("  id: {}", id);

            if let Some(instant) = peer_data.most_recent() {
                info!("    elapsed: {:?}", instant.elapsed());
            }

            info!("    analysis...");
            for x in peer_data.analysis() {
                info!("      {:?}", x);
            }
            info!(
                "    conclusion: {:?}",
                peer_data.analysis().max_by_key(|x| x.1)
            );

            info!("    rx_events...");
            for event in peer_data.rx_events() {
                info!("      {:?}", event);
            }
        }
    }
}
