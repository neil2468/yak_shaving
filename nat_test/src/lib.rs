use tracing::Level;

/// May panic
pub fn setup_tracing() {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to setup tracing");
}
