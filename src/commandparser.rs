use std::net::SocketAddr;

use crate::connections::ConnectionMsg;
use crate::error::Result;
use crate::tui::{Event, StdoutMsg, TuiEventSender};

pub enum Command {
    ConnectionMsg(ConnectionMsg),
    Help,
    MissingArgs,
    NotFound,
}

pub fn parse(
    input: &str,
    sender_username: Vec<u8>,
) -> Result<Command> {
    let mut split = input.split(' ');
    let command = split
        .next()
        .expect("the first split is always there even if there is no value");

    let mut args = split;

    match command {
        "/help" => {
            Ok(Command::Help)
        }
        "/connect" => {
            let Some(recipient) = args.next() else { return Ok(Command::MissingArgs); };
            let Some(chat_name) = args.next() else { return Ok(Command::MissingArgs); };
            let recipient = recipient.parse::<SocketAddr>()?;
            let chat_name = chat_name.to_string();
            let con_msg = ConnectionMsg::CreateConnection(recipient, chat_name);
            Ok(Command::ConnectionMsg(con_msg))
        }
        "/msg" => {
            let Some(chat_name) = args.next() else { return Ok(Command::MissingArgs); };
            let Some(msg) = args.next() else { return Ok(Command::MissingArgs); };
            let chat_name = chat_name.to_string();
            let msg = msg.as_bytes().to_vec();

            let con_msg = ConnectionMsg::Outgoing {
                msg,
                sender: sender_username,
                chat_name,
            };
            Ok(Command::ConnectionMsg(con_msg))
        }
        "/create-group" => {
            let Some(group_name) = args.next() else { return Ok(Command::MissingArgs); };
            let mut participants = Vec::new();

            for arg in args {
                participants.push(arg.to_string());
            }

            let con_msg = ConnectionMsg::CreateGroup(group_name.to_string(), participants);
            Ok(Command::ConnectionMsg(con_msg))
        }
        "/msg-group" => {
            let Some(group_name) = args.next() else { return Ok(Command::MissingArgs); };
            let Some(msg) = args.next() else { return Ok(Command::MissingArgs); };
            let group_name = group_name.to_string();
            let msg = msg.as_bytes().to_vec();

            let con_msg = ConnectionMsg::OutgoingGroup {
                msg,
                sender: sender_username,
                group_name,
            };
            Ok(Command::ConnectionMsg(con_msg))
        }
        "/broadcast" => {
            let Some(msg) = args.next() else { return Ok(Command::MissingArgs); };
            let msg = msg.as_bytes().to_vec();
            let con_msg = ConnectionMsg::Broadcast {
                msg,
                sender: sender_username,
            };
            Ok(Command::ConnectionMsg(con_msg))
        }
        _ => Ok(Command::NotFound),
    }
}
