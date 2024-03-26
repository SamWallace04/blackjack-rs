use std::{io::Read, io::Write};

use blackjack_shared::web_socket::*;
use tungstenite::{Message, WebSocket};

pub fn send_request<S>(request: BlackjackRequest, socket: &mut WebSocket<S>)
where
    S: Read + Write,
{
    let json = serde_json::to_string(&request).unwrap();
    socket.send(Message::Text(json)).unwrap()
}

pub fn send_request_and_wait_for_response<S>(
    request: BlackjackRequest,
    socket: &mut WebSocket<S>,
) -> String
where
    S: Read + Write,
{
    send_request(request, socket);

    let msg = wait_for_message(socket);

    match msg {
        msg @ Message::Text(_) => msg.into_text().unwrap(),
        Message::Close(_) => "CLOSE".to_string(),
        _ => String::new(),
    }
}

pub fn wait_for_message<S>(socket: &mut WebSocket<S>) -> Message
where
    S: Read + Write,
{
    let mut msg: Message;
    loop {
        msg = socket.read().expect("Error reading message");
        if msg.is_text() {
            break;
        }
    }
    msg
}
