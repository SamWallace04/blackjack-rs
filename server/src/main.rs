mod card;
mod thread_pool;

use core::panic;
use std::net::TcpListener;

use color_eyre::eyre::Result;

use crate::{card::draw_cards, thread_pool::*};
use blackjack_shared::{
    player::{Player, PlayerType},
    web_socket::*,
};

use tungstenite::{accept, Message};

fn main() -> Result<()> {
    color_eyre::install()?;
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(8);

    let mut handles = vec![];

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            println!("Incoming connection.");

            let handle = pool.execute(move || {
                // First time initialisation.
                let mut web_socket = accept(stream).unwrap();

                let player = Player {
                    player_type: PlayerType::Human,
                    hand: draw_cards(2),
                    hand_value: 0,
                    chips: 500,
                    current_bet: 0,
                };

                loop {
                    // The main loop for handling the connection and communication.
                    let msg = web_socket.read().unwrap();
                    let resp = handle_message(msg);

                    if resp.action == WebSocketAction::Close {
                        web_socket.close(None).unwrap_or_else(|err| {
                            println!("{}", err);
                        });
                        break;
                    }

                    send_request(resp, &mut web_socket);
                }
            });

            handles.push(handle);
        } else {
            println!("Unsuccessful connection made.");
        }
    }

    drop(listener);
    Ok(())
}

fn handle_message(msg: Message) -> WebSocketRequest {
    println!("Recieved message {}", msg);
    match msg {
        msg @ Message::Text(_) => {
            let text = msg.into_text().unwrap();
            let request: WebSocketRequest =
                serde_json::from_str(&text).expect("Request to be of type WebSocketRequest");

            match request.command {
                //TODO: Return the player object.
                RequestCommand::Start => WebSocketRequest {
                    action: WebSocketAction::Send,
                    command: RequestCommand::Info,
                    message: request.message,
                },
                _ => panic!(),
            }
        }
        Message::Close(_) => WebSocketRequest {
            action: WebSocketAction::Close,
            command: RequestCommand::Info,
            message: String::new(),
        },
        _ => WebSocketRequest {
            action: WebSocketAction::None,
            command: RequestCommand::Info,
            message: String::new(),
        },
    }
}

// loop {
//     let buf_reader = BufReader::new(&mut stream);
//
//     if buf_reader.buffer().is_empty() {
//         return;
//     }
//     let received: String = buf_reader
//         .lines()
//         .map(|result| result.unwrap())
//         .take_while(|line| !line.is_empty())
//         .collect();
//
//     println!("Request: {}", received);
//
//     let card = Card {
//         rank: rank_from_int(1),
//         suit: suit_from_int(2),
//     };
//
//     let serialised = serde_json::to_string(&card).unwrap();
//     println!("sending {}", serialised);
//     stream.write_all(serialised.as_bytes()).unwrap();
//     stream.flush().unwrap();
// }
