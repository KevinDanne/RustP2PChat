use std::net::{SocketAddr, TcpListener, TcpStream};

use crate::connections::{ConnectionMsg, ConnectionMsgSender};
use crate::error::Result;
use crate::message::Message;
use crate::tui::{Event, StdoutMsg, TuiEventSender, Color};

// 1. Accept incoming connectioin
// 2. Identify connection (e.g recieve a name)
// 3. Pass the connection on the "global" list of connection

pub fn listen(
    addr: &str,
    con_sender: ConnectionMsgSender,
    tui_event_sender: TuiEventSender,
) -> Result<()> {
    tui_event_sender.send(Event::User(StdoutMsg::new(format!(
        "listening on address: {addr}"
    ))));
    let listener = TcpListener::bind(addr)?;

    while let Ok((stream, addr)) = listener.accept() {
        let con_sender = con_sender.clone();
        let tui_event_sender = tui_event_sender.clone();
        std::thread::spawn(move || {
            if let Ok(()) = validate(&addr, tui_event_sender.clone()) {
                connect(stream, addr, con_sender, tui_event_sender).unwrap();
            }
        });
    }

    Ok(())
}

pub fn validate(addr: &SocketAddr, tui_event_sender: TuiEventSender) -> Result<()> {
    tui_event_sender.send(Event::User(StdoutMsg::new(format!(
        "Validating incoming connection from {addr}"
    ))));
    // TODO enter validation logic
    tui_event_sender.send(Event::User(StdoutMsg::with_color(
        "Connection validated".to_string(),
        Color::Green,
        Color::Black
    )));
    Ok(())
}

pub fn connect(
    mut stream: TcpStream,
    addr: SocketAddr,
    con_sender: ConnectionMsgSender,
    tui_event_sender: TuiEventSender,
) -> Result<()> {
    tui_event_sender.send(Event::User(StdoutMsg::new(format!("Connecting to {addr}"))));

    let writer = stream.try_clone()?;
    // TODO dont clone
    con_sender.send(ConnectionMsg::AcceptedConnection(writer, addr))?;

    // TODO dont clone chat_name every iteration
    loop {
        let message = Message::frame(&mut stream)?;
        let payload = message.to_owned_string()?;
        con_sender.send(ConnectionMsg::Incoming(addr, payload))?;
    }
}
