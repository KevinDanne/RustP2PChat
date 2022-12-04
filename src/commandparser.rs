use std::net::SocketAddr;

use crate::connections::ConnectionMsg;
use crate::error::Result;
use crate::tui::{Event, StdoutMsg, TuiEventSender};

pub fn parse(
    input: &str,
    sender_username: Vec<u8>,
    tui_event_sender: TuiEventSender,
) -> Result<Option<ConnectionMsg>> {
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
            let receiver = split.next().unwrap();
            let receiver = receiver.parse::<SocketAddr>()?;
            let msg = match split.next() {
                Some(msg) => msg.as_bytes().to_vec(),
                None => {
                    tui_event_sender.send(Event::User(StdoutMsg::new(
                        "no message provided".to_string(),
                    )));
                    return Ok(None);
                }
            };
            let con_msg = ConnectionMsg::Outgoing {
                msg,
                sender: sender_username,
                receiver,
            };
            Ok(Some(con_msg))
        }
        "/create-group" => {
            let mut split = args.split(' ');
            let group_name = split.next().unwrap().to_string();
            let mut participants = Vec::new();

            for arg in split {
                let participant = arg.parse::<SocketAddr>()?;
                participants.push(participant);
            }

            let con_msg = ConnectionMsg::CreateGroup(group_name, participants);
            Ok(Some(con_msg))
        }
        "/msg-group" => {
            let mut split = args.splitn(2, ' ');
            let group_name = split.next().unwrap().to_string();
            let msg = match split.next() {
                Some(msg) => msg.as_bytes().to_vec(),
                None => {
                    tui_event_sender.send(Event::User(StdoutMsg::new(
                        "no message provided".to_string(),
                    )));
                    return Ok(None);
                }
            };
            let con_msg = ConnectionMsg::OutgoingGroup {
                msg,
                sender: sender_username,
                group_name,
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
