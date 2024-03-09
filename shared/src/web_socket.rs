use std::{
    io::{Read, Write},
    net::TcpStream,
};

use tungstenite::{stream::MaybeTlsStream, Message, WebSocket};

#[derive(Debug, PartialEq)]
pub enum WebSocketAction {
    Send,
    Close,
    None,
}

#[derive(Debug)]
pub struct WebSocketRequest {
    pub action: WebSocketAction,
    pub message: String,
}

impl WebSocketRequest {
    pub fn new(action: WebSocketAction, message: String) -> WebSocketRequest {
        WebSocketRequest { action, message }
    }

    pub fn send_request<S>(&self, socket: &mut WebSocket<S>)
    where
        S: Read + Write,
    {
        match self.action {
            WebSocketAction::Send => socket
                .send(Message::Text(self.message.to_owned().into()))
                .unwrap(),
            WebSocketAction::Close => socket
                .close(None)
                .unwrap_or_else(|error| println!("{}", error)),
            WebSocketAction::None => {}
        };
    }

    pub fn send_request_and_wait_for_response<S>(&self, socket: &mut WebSocket<S>) -> String
    where
        S: Read + Write,
    {
        self.send_request(socket);

        let resp;

        loop {
            let msg = socket.read().expect("Error reading message");
            if msg.is_text() {
                match msg {
                    msg @ Message::Text(_) => resp = msg.into_text().unwrap(),
                    Message::Close(_) => resp = "CLOSE".to_string(),
                    _ => resp = String::new(),
                };
                break;
            }
        }
        resp
    }
}
