use clap::Parser;
use std::{
    net::{SocketAddr, UdpSocket},
    str::{self, FromStr},
    thread,
};
use tracing::{info, Level};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(long, default_value_t = 4000)]
    port1: u16,

    #[arg(long, default_value_t = 4001)]
    port2: u16,

    #[arg(long, default_value_t = 4002)]
    port3: u16,
}

fn main() -> anyhow::Result<()> {
    // Set tracing level
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args = Args::parse();
    info!("Args: {:?}", args);

    let mut join_handles = vec![];

    for port in [args.port1, args.port2, args.port3] {
        let addr = SocketAddr::from(([0, 0, 0, 0], port));
        join_handles.push(thread::spawn(move || -> anyhow::Result<()> {
            info!("Binding to {:?}", addr);
            let socket = UdpSocket::bind(addr)?;
            let mut buf = [0; 1024];
            loop {
                let (rx_count, src_addr) = socket.recv_from(&mut buf)?;

                let mut log = vec![];
                log.push(format!("Rx'ed from {:?} via {:?}", src_addr, addr));

                log.push(match str::from_utf8(&buf[..rx_count]) {
                    Ok(buf_string) => match SocketAddr::from_str(&buf_string) {
                        Ok(buf_addr) => format!(
                            "{} vs {}: {}. Msg: {}",
                            src_addr.port(),
                            buf_addr.port(),
                            src_addr.port() == buf_addr.port(),
                            buf_string,
                        ),
                        Err(_) => format!("String '{}''", buf_string),
                    },
                    Err(_) => format!("Bytes {:?}", &buf[..rx_count]),
                });

                info!("\n{}", log.join("\n"));
            }
        }));
    }

    for h in join_handles {
        h.join().unwrap()?;
    }

    Ok(())
}
