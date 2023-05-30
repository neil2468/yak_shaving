use clap::Parser;
use std::{
    net::{SocketAddr, UdpSocket},
    str, thread,
};
use tracing::{info, warn, Level};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {}

const START_PORT: u16 = 5000;
const END_PORT: u16 = 5999;

fn main() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    info!("Args: {:?}", args);

    let mut join_handles = vec![];

    info!("Binding to ports {START_PORT}..={END_PORT}");

    for port in START_PORT..=END_PORT {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        join_handles.push(thread::spawn(move || -> anyhow::Result<()> {
            let socket = UdpSocket::bind(addr)?;
            assert_eq!(port, socket.local_addr().unwrap().port());

            let mut buf = [0; 1024];

            loop {
                let (rx_count, src_addr) = socket.recv_from(&mut buf)?;

                match str::from_utf8(&buf[..rx_count]) {
                    Ok(buf_string) => info!(
                        "Rx on {}: src_port = {}, msg = {}",
                        port,
                        src_addr.port(),
                        buf_string
                    ),
                    Err(e) => warn!("Failed to decode to string: {e}"),
                }
            }
        }));
    }

    for h in join_handles {
        h.join().unwrap()?;
    }

    Ok(())
}
