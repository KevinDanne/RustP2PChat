use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::thread;

use crate::server;

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
        group_name: String
    },
    Broadcast {
        msg: Vec<u8>,
        sender: Vec<u8>,
    }
}

pub fn handle_all_connections(sender: mpsc::Sender<ConnectionMsg>, receiver: mpsc::Receiver<ConnectionMsg>) {
    let mut connections = HashMap::new();
    let mut groups = HashMap::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            ConnectionMsg::CreateConnection(addr) => {
                let stream = match TcpStream::connect(&addr) {
                    Ok(stream) => stream,
                    Err(err) => {
                        eprintln!("Error while connecting to socket {err}");
                        continue;
                    }
                };

                let sender_clone = sender.clone();
                thread::spawn(move || {
                    server::connect(stream, addr, sender_clone).unwrap();
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
                    eprintln!("No chat found for address {addr}");
                    continue;
                }
                println!("{payload}");
            }
            ConnectionMsg::Outgoing {
                msg,
                sender,
                receiver
            } => {
                let Some(stream) = connections.get_mut(&receiver) else {
                    eprintln!("No chat found for address {receiver}");
                    continue;
                };
                stream.write_all(&sender);
                stream.write_all(b" > ");
                stream.write_all(&msg);
            }
            ConnectionMsg::OutgoingGroup {
                msg,
                sender,
                group_name
            } => {
                let Some(participants) = groups.get(&group_name) else {
                    eprintln!("No group found with name {group_name}");
                    continue;
                };
                for addr in participants {
                    let Some(stream) = connections.get_mut(addr) else {
                        eprintln!("No connection found for addr {addr}");
                        continue;
                    };
                    stream.write_all(&sender);
                    stream.write_all(b" (GROUP: ");
                    stream.write_all(group_name.as_bytes());
                    stream.write_all(b") > ");
                    stream.write_all(&msg);
                }
            }
            ConnectionMsg::Broadcast {
                msg,
                sender,
            } => {
                for (_, stream) in connections.iter_mut() {
                    stream.write_all(&sender);
                    stream.write_all(b" > ");
                    stream.write_all(&msg);
                }
            }
        }
    }
}
