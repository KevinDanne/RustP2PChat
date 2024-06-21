use std::collections::HashMap;
use std::fs;
use std::sync::mpsc;

use anathema::runtime::{KeyCode, KeyEvent, Runtime};
use anathema::templates::DataCtx;
use anathema::widgets::Value;

use crate::commandparser::{parse, Command};
use crate::connections::ConnectionMsg;

pub use anathema::runtime::Event;
pub use anathema::display::Color;
pub type TuiEventSender = mpsc::Sender<Event<StdoutMsg>>;

pub struct StdoutMsg {
    pub msg: String,
    pub foreground: Color,
    pub background: Color,
}

impl From<StdoutMsg> for Value {
    fn from(value: StdoutMsg) -> Self {
        let mut hm = HashMap::new();
        hm.insert("msg".to_string(), value.msg.into());
        hm.insert("foreground".to_string(), value.foreground.into());
        hm.insert("background".to_string(), value.background.into());

        Self::Map(hm)
    }
}

impl StdoutMsg {
    pub fn new(msg: String) -> Self {
        Self {
            msg,
            foreground: Color::Reset,
            background: Color::Reset
        }
    }

    pub fn with_foreground(msg: String, foreground: Color) -> Self {
        Self {
            msg,
            foreground,
            background: Color::Reset
        }
    }

    pub fn with_background(msg: String, background: Color) -> Self {
        Self {
            msg,
            foreground: Color::Reset,
            background
        }
    }

    pub fn with_color(msg: String, foreground: Color, background: Color) -> Self {
        Self {
            msg,
            foreground,
            background
        }
    }
}

pub struct Tui {
    runtime: Runtime<StdoutMsg>,
}

impl Tui {
    pub fn new() -> Self {
        Self {
            runtime: Runtime::new(),
        }
    }

    pub fn sender(&self) -> mpsc::Sender<Event<StdoutMsg>> {
        self.runtime.sender()
    }

    pub fn start(self, con_sender: mpsc::Sender<ConnectionMsg>, username: &str) {
        let template =
            fs::read_to_string("./templates/main.tiny").expect("Cant read template.tiny");
        let mut ctx = DataCtx::empty();
        let messages: Vec<StdoutMsg> = Vec::new();
        ctx.set("input", "");
        ctx.set("messages", messages);

        self.runtime.start(template, ctx, |event, _root, ctx, tx| {
            if event.ctrl_c() {
                tx.send(Event::Quit).unwrap();
            }

            if let Event::Key(KeyEvent { code, .. }) = event {
                let input = ctx.get_string_mut("input").expect("No input found in context");
                match code {
                    KeyCode::Char(c) => input.push(c),
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Enter => {
                        match parse(input, username.as_bytes().to_vec()) {
                            Ok(Command::ConnectionMsg(msg)) => {
                                if let Err(err) = con_sender.send(msg) {
                                    tx.send(Event::User(StdoutMsg::new(format!(
                                        "Error while sending message ton connection handler via mpsc: {err}"
                                    ))));
                                    return;
                                }
                            }
                            Ok(Command::NotFound) => {
                                tx.send(Event::User(StdoutMsg::new("Command not found. Use /help for details".to_string())));
                                return;
                            }
                            Ok(Command::MissingArgs) => {
                                tx.send(Event::User(StdoutMsg::new("Missing params. Use /help for details".to_string())));
                                return;
                            }
                            Ok(Command::Help) => {
                                tx.send(Event::User(StdoutMsg::new("
                                /help -> Print help message\n
                                /connect <IP:PORT> <CHAT_NAME> -> Create new chat\n
                                /msg <CHAT_NAME> <MSG> -> Send message to chat\n
                                /create-group <GROUP_NAME> <CHAT_NAME...> -> Create a group with the specified chats\n
                                /msg-group <GROUP_NAME> <MSG> -> Send message to group\n
                                /broadcast <MSG> -> Send message to all chats\n
                                ".to_string())));
                            }
                            Err(e) => {
                                tx.send(Event::User(StdoutMsg::new(format!(
                                    "{e}"
                                ))));
                                return;
                            }
                        }
                        input.clear();
                    }
                    _ => return
                }
            }

            if let Event::User(msg) = event {
                let messages = ctx.get_list_mut("messages").expect("No messages found in context");
                messages.push(msg.into());
            }
        }).unwrap();
    }
}
