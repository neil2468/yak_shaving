use tracing::Level;

#[derive(Debug)]
pub enum Message {
    UpdateReq,
}

pub fn setup_tracing() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}
