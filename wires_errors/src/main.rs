use std::io::{Read, Seek, SeekFrom};

use serde::{Deserialize, Serialize};
use wires_errors::*;

fn main() -> anyhow::Result<()> {
    // let f = Foo {
    //     name: String::from("Neil"),
    //     age: 48,
    // };

    let f = foo();

    let s = serde_json::to_string(&f)?;

    println!("{s}");

    // // Create fake "file"
    // let mut c = std::io::Cursor::new(Vec::new());

    // ciborium::into_writer(&f, c.clone())?;

    // // Read the "file's" contents into a vector
    // let mut out = Vec::new();
    // c.seek(SeekFrom::Start(0)).unwrap();
    // c.read_to_end(&mut out).unwrap();
    // println!("{:x?}", out);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Foo {
    name: String,
    age: u32,
}
