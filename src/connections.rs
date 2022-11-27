use std::collections::HashMap;
use std::io::Write;
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;

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
    NewConnection(TcpStream, SocketAddr, String),
    Incoming(SocketAddr, String),
    Outgoing {
        msg: Vec<u8>,
        sender: Vec<u8>,
        recipient: SocketAddr,
    },
    Broadcast {
        msg: Vec<u8>,
        sender: Vec<u8>,
    }
}

pub fn handle_all_connections(receiver: mpsc::Receiver<ConnectionMsg>) {
    let mut connections = HashMap::new();

    while let Ok(msg) = receiver.recv() {
        match msg {
            ConnectionMsg::NewConnection(stream, addr, username) => {
                println!("new connection validated: {addr}");
                connections.insert(addr, (stream, username));
            }
            ConnectionMsg::Incoming(addr, payload) => {
                let Some((_, username)) = connections.get(&addr) else { continue };
                println!("{username} > {payload}");
            }
            ConnectionMsg::Outgoing {
                msg,
                sender,
                recipient,
            } => {
                let Some((recipient, _)) = connections.get_mut(&recipient) else { continue };
                recipient.write_all(&sender);
                recipient.write_all(b" > ");
                recipient.write_all(&msg);
            }
            ConnectionMsg::Broadcast {
                msg,
                sender,
            } => {
                for (_, (recipient, _)) in connections.iter_mut() {
                    recipient.write_all(&sender);
                    recipient.write_all(b" > ");
                    recipient.write_all(&msg);
                }
            }
        }
    }
}
