mod attribure;
mod message;

use std::{
    io::{Read, Write},
    net::{TcpStream, UdpSocket},
};

use bytes::{BufMut, Bytes, BytesMut};
use rand::{self, random};

use crate::{
    attribure::{Attribute, AttributeType},
    message::{Message, MessageClass, MessageMethod},
};

const SERVER: &str = "stun.mit.de:3478";

fn main() -> anyhow::Result<()> {
    simple_logger::SimpleLogger::new().env().init().unwrap();
    log::info!("Started");

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(SERVER)?;

    let msg = Message {
        method: MessageMethod::Binding,
        class: MessageClass::Request,
        id: random(),
        attributes: Vec::new(),
    };

    let msg = msg.bytes();
    log::trace!("msg ({} bytes): {}", msg.len(), hex::encode(&msg));

    socket.send(&msg)?;

    let mut buf = [0; 1024];
    let len = socket.recv(&mut buf)?;
    log::info!("Received {} bytes: {}", len, hex::encode(&buf[..len]));

    log::info!("Finished");
    Ok(())
}
