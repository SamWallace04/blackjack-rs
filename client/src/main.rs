mod web_socket;

use crate::web_socket::*;

use blackjack_shared::web_socket::*;
use color_eyre::eyre::Result;
use tungstenite::connect;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let http_client = reqwest::Client::new();
    let res_json = http_client
        .post("http://127.0.0.1:8000/register")
        .json(&RegisterRequest { user_id: 1 })
        .send()
        .await?
        .text()
        .await?;

    let res: RegisterResponse = serde_json::from_str(res_json.as_str())?;

    let (mut socket, _) = connect(Url::parse(&res.url).unwrap()).expect("Can't connect");

    println!("Connected to the server");

    let req = BlackjackRequest {
        action: RequestAction::Send,
        command: RequestCommand::Start,
        message: "Hello".to_string(),
    };

    let res = send_request_and_wait_for_response(req, &mut socket);
    println!("res: {}", res);

    let new_req = BlackjackRequest {
        action: RequestAction::Send,
        command: RequestCommand::Start,
        message: "World!".to_string(),
    };

    let res2 = send_request_and_wait_for_response(new_req, &mut socket);
    println!("res2: {}", res2);

    let close_req = BlackjackRequest {
        action: RequestAction::Close,
        command: RequestCommand::Info,
        message: String::new(),
    };

    send_request(close_req, &mut socket);
    Ok(())
}
