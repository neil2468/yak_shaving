use std::net::SocketAddr;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
enum Message {
    UpdateReq(SocketAddr),
    LookupReq(String),
    LookupRes(String),
    PingReq,
    PingRes,
}

fn main() {
    let m = Message::Update("127.0.0.1:8080".parse::<SocketAddr>().unwrap());
    let serialized = serde_json::to_string(&m).unwrap();
    println!("{serialized}");
    println!("{:x?}", serialized.as_bytes());

    let m = Message::Ping;
    let serialized = serde_json::to_string(&m).unwrap();
    println!("{serialized}");
    println!("{:x?}", serialized.as_bytes());
}
