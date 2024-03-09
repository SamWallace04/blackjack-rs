use blackjack_shared::web_socket::*;
use color_eyre::eyre::Result;
use tungstenite::connect;
use url::Url;

fn main() -> Result<()> {
    color_eyre::install()?;

    let (mut socket, _) =
        connect(Url::parse("ws://localhost:7878/socket").unwrap()).expect("Can't connect");

    println!("Connected to the server");

    let req = WebSocketRequest {
        action: WebSocketAction::Send,
        command: RequestCommand::Start,
        message: "Hello".to_string(),
    };

    let res = send_request_and_wait_for_response(req, &mut socket);
    println!("res: {}", res);

    let new_req = WebSocketRequest {
        action: WebSocketAction::Send,
        command: RequestCommand::Start,
        message: "World!".to_string(),
    };

    let res2 = send_request_and_wait_for_response(new_req, &mut socket);
    println!("res2: {}", res2);

    let close_req = WebSocketRequest {
        action: WebSocketAction::Close,
        command: RequestCommand::Info,
        message: String::new(),
    };

    send_request(close_req, &mut socket);
    Ok(())
}
