use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use tungstenite::{Message, WebSocket};

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum WebSocketAction {
    Send,
    Close,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestCommand {
    Start,
    StartTurn,
    Action,
    End,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketRequest {
    pub action: WebSocketAction,
    pub command: RequestCommand,
    pub message: String,
}

pub fn send_request<S>(request: WebSocketRequest, socket: &mut WebSocket<S>)
where
    S: Read + Write,
{
    match request.action {
        WebSocketAction::Send => {
            let json = serde_json::to_string(&request).unwrap();
            socket.send(Message::Text(json)).unwrap()
        }
        WebSocketAction::Close => socket
            .close(None)
            .unwrap_or_else(|error| println!("{}", error)),
        WebSocketAction::None => {}
    };
}

pub fn send_request_and_wait_for_response<S>(
    request: WebSocketRequest,
    socket: &mut WebSocket<S>,
) -> String
where
    S: Read + Write,
{
    send_request(request, socket);

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
