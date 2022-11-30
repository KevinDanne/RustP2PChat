use std::io::Read;

use crate::error::{Error, Result};

pub struct Message {
    payload: Vec<u8>,
    len: usize,
}

impl Message {
    pub fn frame(reader: &mut impl Read) -> Result<Self> {
        let mut message = Message {
            payload: vec![0u8; 1024],
            len: 0,
        };

        let byte_count = reader.read(&mut message.payload)?;
        if byte_count == 0 {
            return Err(Error::ConnectionClosed);
        }

        message.len = byte_count;

        Ok(message)
    }

    pub fn payload(&self) -> &[u8] {
        &self.payload[..self.len]
    }

    pub fn consume(mut self) -> Vec<u8> {
        self.payload.truncate(self.len);
        self.payload
    }

    pub fn to_owned_string(self) -> Result<String> {
        let s = String::from_utf8(self.consume())?;
        Ok(s)
    }
}
