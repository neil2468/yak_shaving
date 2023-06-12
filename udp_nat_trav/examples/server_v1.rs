//! Prototype Rendezvous Server (version 1) for UDP NAT Hole Punching

mod server;
mod shared;

use crate::server::AlphaManager;
use std::{sync::Arc, time::Duration};
use tokio::time::sleep;
use tracing::info;

// TODO: Characterize if we expect NAT to use a different source IP or port with
// another peer than it did with us.

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    shared::setup_tracing()?;
    info!("Started");

    // Spawn tasks for 'one to many' tests
    let manager = Arc::new(AlphaManager::new());
    let mut join_set = manager.spawn_tasks();

    // Monitor task
    join_set.spawn(monitor_task(manager.clone()));

    // Wait on tasks
    while let Some(res) = join_set.join_next().await {
        let _ = res?; // TODO: this does not seem right
    }

    info!("Finished");
    Ok(())
}

async fn monitor_task(mng: Arc<AlphaManager>) -> anyhow::Result<()> {
    loop {
        sleep(Duration::from_millis(1000)).await; // TODO: magic number

        info!("Data...");

        let data = mng.data();

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

            for (ip, count) in peer_data.ip_stats() {
                info!("    ip: {}, count {}", ip, count);
                for (port, count) in peer_data.port_stats(&ip) {
                    info!("      port: {}, count {}", port, count);
                }
            }
        }
    }
}
