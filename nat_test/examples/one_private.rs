use clap::Parser;
use std::{
    net::{IpAddr, SocketAddr, UdpSocket},
    thread,
    time::Duration,
};
use tracing::{self, info};

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {
    #[arg(long = "server")]
    server_ip: String,
}

const PUBLIC_PORT1: u16 = 4000;
const PUBLIC_PORT2: u16 = 4010;
const MSG: &[u8] = "".as_bytes();

fn main() -> anyhow::Result<()> {
    nat_test::setup_tracing();

    info!("Started");
    let args = Args::parse();
    info!("Args: {:?}", args);

    let public_ip: IpAddr = args.server_ip.parse()?;
    let addr1 = SocketAddr::from((public_ip, PUBLIC_PORT1));
    let addr2 = SocketAddr::from((public_ip, PUBLIC_PORT2));

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_nonblocking(true)?;

    // Send initial packets
    tx(&socket, &addr1)?;
    //    tx(&socket, &addr2)?;

    // Listen to socket and send keep alives to one address
    loop {
        let mut recv_buf = [0; 1024];
        for _ in 0..30 {
            match socket.recv_from(&mut recv_buf) {
                Ok((_, src_addr)) => info!("Rx from {}", src_addr),
                Err(_) => (),
            }
            thread::sleep(Duration::from_millis(100));
        }

        tx(&socket, &addr1)?;
    }
}

fn tx(socket: &UdpSocket, addr: &SocketAddr) -> anyhow::Result<()> {
    info!("Tx to {}", addr);
    socket.send_to(MSG, addr)?;
    Ok(())
}
