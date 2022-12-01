use std::io;
use std::sync::mpsc;
use std::thread;

mod commandparser;
mod connections;
mod error;
mod message;
mod server;

use commandparser::parse;

fn main() {
    println!("Enter listening port number:");
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Error while reading port input");
    let listening_port = input.trim().parse::<u16>().expect("Invalid port number entered");
    let (con_sender, con_receiver) = mpsc::channel();

    {
        let con_sender = con_sender.clone();
        thread::spawn(move || {
            server::listen(&format!("0.0.0.0:{listening_port}"), con_sender).unwrap();
        });
    };

    {
        let con_sender = con_sender.clone();
        thread::spawn(move || {
            connections::handle_all_connections(con_sender, con_receiver);
        });
    }

    let stdin = io::stdin();
    let mut input = String::new();

    println!("enter a username");
    stdin.read_line(&mut input).unwrap();
    let username = input.clone();
    let username = username.trim();

    loop {
        input.clear();
        stdin.read_line(&mut input).unwrap();
        input.pop(); // remove the newline char

        match parse(&input, username.as_bytes().to_vec()) {
            Ok(Some(msg)) => {
                if let Err(err) = con_sender.send(msg) {
                    eprintln!("Error while sending message ton connection handler via mpsc: {err}");
                    continue;
                }
            }
            Ok(None) => continue,
            Err(e) => {
                eprintln!("{e}");
                continue;
            }
        }
    }
}
