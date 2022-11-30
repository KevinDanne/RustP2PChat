use std::net::AddrParseError;
use std::string::FromUtf8Error;
use std::{fmt, io};
use std::sync::mpsc::SendError;

use crate::connections::ConnectionMsg;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    // Connectionw as closed (received zero bytes)
    ConnectionClosed,
    // Io error
    IO(io::Error),
    // UTF8 encoding error
    UTF8(FromUtf8Error),
    // Failed to parse socket address
    AddrParse(AddrParseError),
    // MPSC send error
    MPSCSend(SendError<ConnectionMsg>)
}

impl std::error::Error for Error {}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConnectionClosed => write!(f, "connection closed"),
            Self::IO(err) => write!(f, "{err}"),
            Self::UTF8(err) => write!(f, "{err}"),
            Self::AddrParse(err) => write!(f, "{err}"),
            Self::MPSCSend(err) => write!(f, "{err}"),
        }
    }
}

// -----------------------------------------------------------------------------
//   - From impls -
// -----------------------------------------------------------------------------
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Self::IO(err)
    }
}

impl From<AddrParseError> for Error {
    fn from(err: AddrParseError) -> Self {
        Self::AddrParse(err)
    }
}

impl From<FromUtf8Error> for Error {
    fn from(err: FromUtf8Error) -> Self {
        Self::UTF8(err)
    }
}

impl From<SendError<ConnectionMsg>> for Error {
    fn from(err: SendError<ConnectionMsg>) -> Self {
        Self::MPSCSend(err)
    }
}
