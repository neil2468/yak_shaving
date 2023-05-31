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

const SHORT_DELAY: Duration = Duration::from_millis(50);
const LONG_DELAY: Duration = Duration::from_millis(1000);

fn main() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    info!("Args: {:?}", args);

    // TODO: these API need good knowledge of the docs. Should be clearer.
    an_b1(&args, 4000, 4019, 5000, SHORT_DELAY)?;
    thread::sleep(LONG_DELAY);
    an_b1(&args, 4020, 4029, 5010, LONG_DELAY)?;
    thread::sleep(LONG_DELAY);
    a1_bn(&args, 4030, 5020, 5039, SHORT_DELAY)?;
    thread::sleep(LONG_DELAY);
    a1_bn(&args, 4040, 5040, 5049, LONG_DELAY)?;
    thread::sleep(LONG_DELAY);
    a1_b1(&args, 4050, 5060, 20, SHORT_DELAY)?;
    thread::sleep(LONG_DELAY);
    a1_b1(&args, 4060, 5070, 20, SHORT_DELAY)?;
    thread::sleep(LONG_DELAY);
    a1_b1(&args, 4070, 5080, 10, LONG_DELAY)?;

    Ok(())
}

fn tx_packet(mode: &str, socket: &UdpSocket, dst_addr: SocketAddr) -> anyhow::Result<()> {
    let orig_port = socket.local_addr()?.port();
    let buf_string = format!("{}|{}|{}", mode, orig_port, dst_addr.port());
    let buf = buf_string.as_bytes();
    socket.send_to(buf, dst_addr)?;
    Ok(()) // TODO: idiomatic would be to return result of socket send, but need to convert the error?
}

/// Peer A uses single port to send to a single port on Peer B
fn a1_b1(
    args: &Args,
    a_port: u16,
    b_port: u16,
    tries: usize,
    delay: Duration,
) -> anyhow::Result<()> {
    let mode = format!("a1_b1_t{}_d{}", tries, delay.as_millis());

    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], a_port)))?;
    let dst_addr = SocketAddr::from((IpAddr::from_str(&args.server_ip)?, b_port));

    for _ in 0..tries {
        tx_packet(&mode, &socket, dst_addr)?;
        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    Ok(())
}

/// Peer A uses different ports to send to a single port on Peer B
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

        tx_packet(&mode, &socket, dst_addr)?;

        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    Ok(())
}

/// Peer A uses a single port to send to different ports on Peer B
fn a1_bn(
    args: &Args,
    a_port: u16,
    b_start_port: u16,
    b_end_port: u16,
    delay: Duration,
) -> anyhow::Result<()> {
    let mode = format!("a1_bn_d{}", delay.as_millis());

    let socket = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], a_port)))?;

    for dst_port in b_start_port..=b_end_port {
        let dst_addr = SocketAddr::from((IpAddr::from_str(&args.server_ip)?, dst_port));

        tx_packet(&mode, &socket, dst_addr)?;

        if !delay.is_zero() {
            thread::sleep(delay);
        }
    }

    Ok(())
}
