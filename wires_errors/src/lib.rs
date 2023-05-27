use std::{
    fmt::{self, Display, Formatter},
    io,
};

use serde::Serialize;

pub fn foo() -> Result<String, ReadError> {
    // return Err(ReadError {
    //     kind: ReadErrorKind::File(std::io::Error::from(std::io::ErrorKind::NotFound)),
    // });

    return Err(ReadError {
        kind: ReadErrorKind::Parse(ParseError {}),
    });

    return Ok(String::from("abc"));
}

#[derive(Debug, Serialize)]
#[non_exhaustive] // Don't allow creation outside this create
pub struct ReadError {
    kind: ReadErrorKind,
}

impl Display for ReadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "read error")
    }
}

impl std::error::Error for ReadError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match &self.kind {
            // ReadErrorKind::File(e) => Some(e),
            ReadErrorKind::Parse(e) => Some(e),
        }
    }
}

#[derive(Debug, Serialize)]
enum ReadErrorKind {
    // File(io::Error),
    Parse(ParseError),
}

#[derive(Debug, Serialize)]
struct ParseError;

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "failed to parse file on line XX")
    }
}

impl std::error::Error for ParseError {}
