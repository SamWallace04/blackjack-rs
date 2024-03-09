use blackjack_shared::web_socket::{WebSocketAction, WebSocketRequest};
use color_eyre::eyre::Result;
use tungstenite::connect;
use url::Url;

fn main() -> Result<()> {
    color_eyre::install()?;

    let (mut socket, _) =
        connect(Url::parse("ws://localhost:7878/socket").unwrap()).expect("Can't connect");

    println!("Connected to the server");

    let req = WebSocketRequest::new(WebSocketAction::Send, "Hello".to_string());

    let res = req.send_request_and_wait_for_response(&mut socket);
    println!("res: {}", res);

    let new_req = WebSocketRequest::new(WebSocketAction::Send, "World!".to_string());

    let res2 = new_req.send_request_and_wait_for_response(&mut socket);
    println!("res2: {}", res2);

    let close = WebSocketRequest::new(WebSocketAction::Close, String::new());

    close.send_request(&mut socket);
    Ok(())
}
