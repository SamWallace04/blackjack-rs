mod player_input;
mod web_socket;

use std::{thread, time::Duration};

use crate::{player_input::get_user_input, web_socket::*};

use blackjack_shared::web_socket::*;
use color_eyre::eyre::Result;
use tungstenite::connect;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Please enter your username:");
    let my_user_name = get_user_input();

    let http_client = reqwest::Client::new();
    let res_json = http_client
        .post("http://127.0.0.1:8000/register")
        .json(&RegisterRequest {
            user_name: my_user_name.to_string(),
        })
        .send()
        .await?
        .text()
        .await?;

    let res: RegisterResponse = serde_json::from_str(res_json.as_str())?;

    let (mut socket, _) = connect(Url::parse(&res.url).unwrap()).expect("Can't connect");

    println!("Connected to the server");
    let client_id = res.id.clone();

    if res.is_host {
        loop {
            println!("You are the host. Enter 'start' to begin the game.");
            let input = get_user_input();
            if input == "start" {
                http_client
                    .post("http://127.0.0.1:8000/publish")
                    .json(&PublishRequest {
                        trigger: PublishTrigger::StartTurn {
                            active_client_id: client_id.clone(),
                            user_name: my_user_name.clone(),
                        },
                    })
                    .send()
                    .await?;
                break;
            }
        }
    } else {
        println!("Waiting for the host to start the game...");
        wait_for_message(&mut socket);
    }

    println!("The game is starting.");
    //Check the start response. If the username matches ours then start the game.
    //Otherwise wait in a loop checking messages and updating the output as required.
    //Once a start message with our name has been send start playing the game.
    let mut current_player_name = String::new();
    loop {
        println!("Waiting for our turn...");
        let message = wait_for_message(&mut socket);
        println!("Message: {}", message.clone().into_text().unwrap());
        let request: PublishRequest = serde_json::from_str(message.into_text().unwrap().as_str())?;

        match request.trigger {
            PublishTrigger::StartTurn {
                active_client_id,
                user_name,
            } => {
                current_player_name = user_name.clone();
                if active_client_id.to_lowercase() == client_id.to_lowercase() {
                    println!("It's our turn!");
                    break;
                }
                println!("It's {}'s turn.", current_player_name);
            }
            PublishTrigger::CardDrawn => {
                println!("A card has been drawn by {}.", current_player_name);
            }
            _ => {
                println!("It's not our turn yet.");
            }
        }
    }

    println!("Waiting");
    thread::sleep(Duration::from_secs(5));

    send_request(
        BlackjackRequest {
            command: RequestCommand::EndTurn,
            message: None,
        },
        &mut socket,
    );

    // let req = BlackjackRequest {
    //     command: RequestCommand::Start,
    //     message: "Hello".to_string(),
    // };
    //
    // let res = send_request_and_wait_for_response(req, &mut socket);
    // println!("res: {}", res);
    //
    // let new_req = BlackjackRequest {
    //     command: RequestCommand::Start,
    //     message: "World!".to_string(),
    // };
    //
    // let res2 = send_request_and_wait_for_response(new_req, &mut socket);
    // println!("res2: {}", res2);
    //
    let close_req = BlackjackRequest {
        command: RequestCommand::Close,
        message: None,
    };

    send_request(close_req, &mut socket);

    //TODO: Run the game locally broadcasting the cards that are drawn?
    //Otherwise send a request per action and broadcast the result? <----
    Ok(())
}
