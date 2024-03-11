use std::{io::Read, io::Write};

use blackjack_shared::web_socket::{BlackjackRequest, RequestAction};
use tungstenite::{Message, WebSocket};

pub fn send_request<S>(request: BlackjackRequest, socket: &mut WebSocket<S>)
where
    S: Read + Write,
{
    match request.action {
        RequestAction::Send => {
            let json = serde_json::to_string(&request).unwrap();
            socket.send(Message::Text(json)).unwrap()
        }
        RequestAction::Close => socket
            .close(None)
            .unwrap_or_else(|error| println!("{}", error)),
        RequestAction::None => {}
    };
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
