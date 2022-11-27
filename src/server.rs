use std::io::{Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;

use crate::connections::ConnectionMsg;
use crate::error::Result;
use crate::message::Message;

// 1. Accept incoming connectioin
// 2. Identify connection (e.g recieve a name)
// 3. Pass the connection on the "global" list of connection

pub fn listen(addr: &str, sender: mpsc::Sender<ConnectionMsg>) -> Result<()> {
    eprintln!("listening on address: {addr}");
    let mut listener = TcpListener::bind(addr)?;

    while let Ok((stream, addr)) = listener.accept() {
        let sender = sender.clone();
        std::thread::spawn(move || validate(stream, addr, sender));
    }

    Ok(())
}

fn validate(
    mut stream: TcpStream,
    addr: SocketAddr,
    sender: mpsc::Sender<ConnectionMsg>,
) -> Result<()> {
    eprintln!("Incoming connection from {addr}");

    stream.write(b"provide a username\n");
    let username = Message::frame(&mut stream)?.to_owned_string()?;

    let writer = stream.try_clone()?;

    sender.send(ConnectionMsg::NewConnection(writer, addr, username));

    loop {
        let message = Message::frame(&mut stream)?;
        let payload = message.to_owned_string()?;
        sender.send(ConnectionMsg::Incoming(addr, payload));
    }

    // If this is valid the pass the connection on
    // For now we assume that they are all valid
    // tx.send((stream, addr));
    Ok(())
}
