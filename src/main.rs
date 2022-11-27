use std::io::{self, Write};
use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

mod commandparser;
mod connections;
mod error;
mod message;
mod server;

use commandparser::parse;
use connections::ConnectionMsg;

enum State {
    NoUsername,
    Username(String),
    PrepMessage(SocketAddr, String),
}

fn main() {
    let (con_sender, con_receiver) = mpsc::channel();

    let listener_handle = {
        let con_sender = con_sender.clone();
        thread::spawn(move || {
            server::listen("127.0.0.1:54321", con_sender);
        })
    };

    let connections_handle = thread::spawn(move || {
        connections::handle_all_connections(con_receiver);
    });

    let stdin = io::stdin();
    let mut input = String::new();

    println!("enter a username");
    stdin.read_line(&mut input).unwrap();
    let username = input.clone();

    let mut state = State::NoUsername;

    loop {
        input.clear();
        stdin.read_line(&mut input).unwrap();
        input.pop(); // remove the newline char

        match parse(&input, username.as_bytes().to_vec()) {
            Ok(Some(msg)) => {
                con_sender.send(msg);
            }
            Ok(None) => continue,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        }
    }

    // -----------------------------------------------------------------------------
    //   - Join all threads -
    // -----------------------------------------------------------------------------
    listener_handle.join().unwrap();
    connections_handle.join().unwrap();
}
