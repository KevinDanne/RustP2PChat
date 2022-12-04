use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;

use crate::server;
use crate::tui::{Event, StdoutMsg, TuiEventSender};

pub type ConnectionMsgSender = mpsc::Sender<ConnectionMsg>;
pub type ConnectionMsgReceiver = mpsc::Receiver<ConnectionMsg>;

pub struct Connection {
    stream: TcpStream,
    addr: SocketAddr,
    username: String,
}

impl Connection {
    pub fn new(username: String, stream: TcpStream, addr: SocketAddr) -> Self {
        Self {
            stream,
            username,
            addr,
        }
    }
}

pub enum ConnectionMsg {
    CreateConnection(SocketAddr),
    AcceptedConnection(TcpStream, SocketAddr),
    CreateGroup(String, Vec<SocketAddr>),
    Incoming(SocketAddr, String),
    Outgoing {
        msg: Vec<u8>,
        sender: Vec<u8>,
        receiver: SocketAddr,
    },
    OutgoingGroup {
        msg: Vec<u8>,
        sender: Vec<u8>,
        group_name: String,
    },
    Broadcast {
        msg: Vec<u8>,
        sender: Vec<u8>,
    },
}

pub fn handle_all_connections(
    con_sender: ConnectionMsgSender,
    con_receiver: ConnectionMsgReceiver,
    tui_event_sender: TuiEventSender,
) {
    let mut connections = HashMap::new();
    let mut groups = HashMap::new();

    while let Ok(msg) = con_receiver.recv() {
        match msg {
            ConnectionMsg::CreateConnection(addr) => {
                let stream = match TcpStream::connect(&addr) {
                    Ok(stream) => stream,
                    Err(err) => {
                        tui_event_sender.send(Event::User(StdoutMsg::new(format!(
                            "Error while connecting to socket {err}"
                        ))));
                        continue;
                    }
                };

                let con_sender = con_sender.clone();
                let tui_event_sender = tui_event_sender.clone();
                thread::spawn(move || {
                    server::connect(stream, addr, con_sender, tui_event_sender).unwrap();
                });
            }
            ConnectionMsg::AcceptedConnection(stream, addr) => {
                connections.insert(addr, stream);
            }
            ConnectionMsg::CreateGroup(name, participants) => {
                groups.insert(name, participants);
            }
            ConnectionMsg::Incoming(addr, payload) => {
                if let None = connections.get(&addr) {
                    tui_event_sender.send(Event::User(StdoutMsg::new(format!(
                        "No chat found for address {addr}"
                    ))));
                    continue;
                }
                tui_event_sender.send(Event::User(StdoutMsg::new(payload)));
            }
            ConnectionMsg::Outgoing {
                msg,
                sender,
                receiver,
            } => {
                let Some(stream) = connections.get_mut(&receiver) else {
                    tui_event_sender.send(Event::User(StdoutMsg::new(format!(
                        "No chat found for address {receiver}"
                    ))));
                    continue;
                };
                stream.write_all(&sender);
                stream.write_all(b" > ");
                stream.write_all(&msg);

                let Ok(sender) = String::from_utf8(sender) else { return };
                let Ok(msg) = String::from_utf8(msg) else { return };
                tui_event_sender.send(Event::User(StdoutMsg::new(format!("{sender} > {msg}"))));
            }
            ConnectionMsg::OutgoingGroup {
                msg,
                sender,
                group_name,
            } => {
                let Some(participants) = groups.get(&group_name) else {
                    tui_event_sender.send(Event::User(StdoutMsg::new(format!(
                        "No group found with name {group_name}"
                    ))));
                    continue;
                };
                for addr in participants {
                    let Some(stream) = connections.get_mut(addr) else {
                        tui_event_sender.send(Event::User(StdoutMsg::new(format!(
                            "No connection found for addr {addr}"
                        ))));
                        continue;
                    };
                    stream.write_all(&sender);
                    stream.write_all(b" (GROUP: ");
                    stream.write_all(group_name.as_bytes());
                    stream.write_all(b") > ");
                    stream.write_all(&msg);
                }
            }
            ConnectionMsg::Broadcast { msg, sender } => {
                for (_, stream) in connections.iter_mut() {
                    stream.write_all(&sender);
                    stream.write_all(b" > ");
                    stream.write_all(&msg);
                }
            }
        }
    }
}
