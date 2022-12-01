use std::io::{self, BufRead};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::mpsc;

use crate::connections::ConnectionMsg;
use crate::error::Result;
use crate::message::Message;

// 1. Accept incoming connectioin
// 2. Identify connection (e.g recieve a name)
// 3. Pass the connection on the "global" list of connection

pub fn listen(addr: &str, sender: mpsc::Sender<ConnectionMsg>) -> Result<()> {
    println!("listening on address: {addr}");
    let listener = TcpListener::bind(addr)?;

    while let Ok((stream, addr)) = listener.accept() {
        let sender = sender.clone();
        std::thread::spawn(move || {
            if let Ok(()) = validate(&addr) {
                connect(stream, addr, sender).unwrap();
            }
        });
    }

    Ok(())
}

pub fn validate(
    addr: &SocketAddr,
) -> Result<()> {
    println!("Validating incoming connection from {addr}");
    // TODO enter validation logic
    println!("Connection validated");
    Ok(())
}

pub fn connect(mut stream: TcpStream, addr: SocketAddr, sender: mpsc::Sender<ConnectionMsg>) -> Result<()> {
    println!("Connecting to {addr}");
    
    let writer = stream.try_clone()?;
    // TODO dont clone
    sender.send(ConnectionMsg::AcceptedConnection(writer, addr))?;

    // TODO dont clone chat_name every iteration
    loop {
        let message = Message::frame(&mut stream)?;
        let payload = message.to_owned_string()?;
        sender.send(ConnectionMsg::Incoming(addr, payload))?;
    }
}
