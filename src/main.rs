use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;

mod commandparser;
mod connections;
mod error;
mod message;
mod server;
mod tui;

use tui::Tui;

fn main() {
    let mut stdout = io::stdout();

    print!("Enter listening port number: ");
    stdout.flush();
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .expect("Error while reading port input");
    let listening_port = input
        .trim()
        .parse::<u16>()
        .expect("Invalid port number entered");

    print!("enter a username: ");
    stdout.flush();
    input.clear();
    io::stdin().read_line(&mut input).unwrap();
    let username = input.trim();

    let (con_sender, con_receiver) = mpsc::channel();

    let tui = Tui::new();

    {
        let con_sender = con_sender.clone();
        let tui_event_sender = tui.sender();
        thread::spawn(move || {
            server::listen(
                &format!("0.0.0.0:{listening_port}"),
                con_sender,
                tui_event_sender,
            )
            .unwrap();
        });
    };

    {
        let con_sender = con_sender.clone();
        let tui_event_sender = tui.sender();
        thread::spawn(move || {
            connections::handle_all_connections(con_sender, con_receiver, tui_event_sender);
        });
    }

    tui.start(con_sender, username);
}
