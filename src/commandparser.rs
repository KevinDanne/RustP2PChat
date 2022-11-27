use std::net::SocketAddr;

use crate::connections::ConnectionMsg;
use crate::error::Result;

fn stupid() -> ! {
    loop {}
}

pub fn parse(input: &str, sender: Vec<u8>) -> Result<Option<ConnectionMsg>> {
    let mut split = input.splitn(2, ' ');
    let command = split
        .next()
        .expect("the first split is always there even if there is no value");

    let Some(args) = split.next() else { return Ok(None) };

    match command {
        "/msg" => {
            let mut split = args.splitn(2, ' ');
            let recipient = split.next().expect("yeye you know the drill");
            let recipient = recipient.parse::<SocketAddr>()?;
            let msg = match split.next() {
                Some(msg) => msg.as_bytes().to_vec(),
                None => {
                    eprintln!("no message provided");
                    return Ok(None);
                }
            };
            let con_msg = ConnectionMsg::Outgoing {
                msg,
                sender,
                recipient,
            };
            Ok(Some(con_msg))
        }
        "/broadcast" => {
            let msg = args.as_bytes().to_vec();
            let con_msg = ConnectionMsg::Broadcast {
                msg,
                sender,
            };
            Ok(Some(con_msg))
        }
        _ => Ok(None),
    }
}
