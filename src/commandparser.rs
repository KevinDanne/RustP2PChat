use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;

use crate::connections::ConnectionMsg;
use crate::error::Result;

pub fn parse(input: &str, sender_username: Vec<u8>) -> Result<Option<ConnectionMsg>> {
    let mut split = input.splitn(2, ' ');
    let command = split
        .next()
        .expect("the first split is always there even if there is no value");

    let Some(args) = split.next() else { return Ok(None) };

    match command {
        "/connect" => {
            let recipient = args.parse::<SocketAddr>()?;
            let con_msg = ConnectionMsg::CreateConnection(recipient);
            Ok(Some(con_msg))
        }
        "/msg" => {
            let mut split = args.splitn(2, ' ');
            let chat_name = split.next().unwrap();
            let chat_name = chat_name.to_string();
            let msg = match split.next() {
                Some(msg) => msg.as_bytes().to_vec(),
                None => {
                    eprintln!("no message provided");
                    return Ok(None);
                }
            };
            let con_msg = ConnectionMsg::Outgoing {
                msg,
                sender: sender_username,
                chat_name,
            };
            Ok(Some(con_msg))
        }
        "/broadcast" => {
            let msg = args.as_bytes().to_vec();
            let con_msg = ConnectionMsg::Broadcast {
                msg,
                sender: sender_username,
            };
            Ok(Some(con_msg))
        }
        _ => Ok(None),
    }
}
