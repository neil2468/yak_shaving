use serde::{Deserialize, Serialize};
use tracing::Level;

pub const ALPHA_PORT_BASE: u16 = 4000;
pub const ALPHA_PORT_COUNT: u16 = 10;
pub const BETA_PORT: u16 = 4010;
pub const BETA_COUNT: usize = 10;
pub const API_PORT: u16 = 4011;

#[derive(Debug, Serialize, Deserialize)]
pub enum Message {
    QueryReq(String), // id#peer_id
    QueryRes(String), //
}

pub fn setup_tracing() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
