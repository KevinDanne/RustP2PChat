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
    AcceptedConnection(TcpStream, SocketAddr, String),
    Incoming(SocketAddr, String, String),
    Outgoing {
        msg: Vec<u8>,
        sender: Vec<u8>,
        chat_name: String,
    },
    Broadcast {
        msg: Vec<u8>,
        sender: Vec<u8>,
    }
}

pub fn handle_all_connections(sender: mpsc::Sender<ConnectionMsg>, receiver: mpsc::Receiver<ConnectionMsg>) {
    let mut connections = HashMap::new();

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
            ConnectionMsg::AcceptedConnection(stream, addr, alias) => {
                connections.insert(alias, (stream, addr));
            }
            ConnectionMsg::Incoming(_addr, chat_name, payload) => {
                if let None = connections.get(&chat_name) {
                    eprintln!("No chat found with name {chat_name}");
                    continue;
                }
                println!("{payload}");
            }
            ConnectionMsg::Outgoing {
                msg,
                sender,
                chat_name,
            } => {
                let Some((stream, _)) = connections.get_mut(&chat_name) else {
                    eprintln!("No chat found with name {chat_name}");
                    eprintln!("{:?}", connections.keys().collect::<Vec<&String>>());
                    continue;
                };
                stream.write_all(&sender);
                stream.write_all(b" > ");
                stream.write_all(&msg);
            }
            ConnectionMsg::Broadcast {
                msg,
                sender,
            } => {
                for (_, (stream, _)) in connections.iter_mut() {
                    stream.write_all(&sender);
                    stream.write_all(b" > ");
                    stream.write_all(&msg);
                }
            }
        }
    }
}
