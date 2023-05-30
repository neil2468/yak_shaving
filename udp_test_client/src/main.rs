use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    str::{self, FromStr},
    thread,
    time::Duration,
};
use tracing::{info, Level};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long = "server")]
    server_ip: String,
}

fn main() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    info!("Args: {:?}", args);

    an_b1(&args, 4000, 4019, 5000, Duration::ZERO)?;
    an_b1(&args, 4020, 4029, 5010, Duration::from_millis(1000))?;

    Ok(())
}

fn tx_packet(mode: &str, socket: UdpSocket, dst_addr: SocketAddr) -> anyhow::Result<()> {
    let orig_port = socket.local_addr()?.port();
    let buf_string = format!("{}|{}|{}", mode, orig_port, dst_addr.port());
    let buf = buf_string.as_bytes();
    socket.send_to(buf, dst_addr)?;
    Ok(()) // TODO: idiomatic would be to return result of socket send, but need to convert the error?
}

fn an_b1(
    args: &Args,
    a_start_port: u16,
    a_end_port: u16,
    b_port: u16,
    delay: Duration,
) -> anyhow::Result<()> {
    let mode = format!("an_b1_d{}", delay.as_millis());
    let dst_addr = SocketAddr::from((IpAddr::from_str(&args.server_ip)?, b_port));

    for orig_port in a_start_port..=a_end_port {
        let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], orig_port)))?;

        tx_packet(&mode, socket, dst_addr)?;

        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    Ok(())
}
