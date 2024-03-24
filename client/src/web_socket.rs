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

    let resp;

    loop {
        let msg = wait_for_message(socket);
        match msg {
            msg @ Message::Text(_) => resp = msg.into_text().unwrap(),
            Message::Close(_) => resp = "CLOSE".to_string(),
            _ => resp = String::new(),
        };
        break;
    }
    resp
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

fn wait_for_start_message<S>(socket: &mut WebSocket<S>)
where
    S: Read + Write,
{
    loop {
        let msg = wait_for_message(socket);
        match msg {
            msg @ Message::Text(_) => {
                let req: BlackjackRequest =
                    serde_json::from_str(msg.into_text().unwrap().as_str()).unwrap();
                if req.command == RequestCommand::Start {
                    break;
                }
            }
            _ => {}
        }
    }
}
