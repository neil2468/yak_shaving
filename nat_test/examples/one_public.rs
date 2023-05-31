use clap::Parser;
use std::{
    net::{SocketAddr, UdpSocket},
    str, thread,
    time::Duration,
};
use tracing::{self, info, warn};

/// Simple program to greet a person
#[derive(Parser, Debug)]
struct Args {}

const PUBLIC_PORT1: u16 = 4000;
const PUBLIC_PORT2: u16 = 4010;
const MSG: &[u8] = "".as_bytes();

fn main() -> anyhow::Result<()> {
    nat_test::setup_tracing();

    info!("Started");

    let mut join_handles = vec![];

    for port in [PUBLIC_PORT1, PUBLIC_PORT2] {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        join_handles.push(thread::spawn(move || -> anyhow::Result<()> {
            let socket = UdpSocket::bind(addr)?;
            assert_eq!(port, socket.local_addr().unwrap().port());

            let mut buf = [0; 1024];

            let mut delay = 10000;

            loop {
                let (rx_count, src_addr) = socket.recv_from(&mut buf)?;

                match str::from_utf8(&buf[..rx_count]) {
                    Ok(buf_string) => info!(
                        "Rx on {}: src_port = {}, msg = {}",
                        port,
                        src_addr.port(),
                        buf_string.trim()
                    ),
                    Err(e) => warn!("Failed to decode to string: {e}"),
                }

                if port == PUBLIC_PORT2 {
                    loop {
                        thread::sleep(Duration::from_millis(delay));
                        info!(
                            "Tx from {} to {} after {}mS",
                            socket.local_addr()?,
                            src_addr,
                            delay
                        );
                        socket.send_to(MSG, src_addr)?;
                        delay = delay + 10000;
                    }
                }
            }
        }));
    }

    for h in join_handles {
        h.join().unwrap()?;
    }

    Ok(())
}
