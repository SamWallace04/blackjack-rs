mod thread_pool;

use std::net::TcpListener;

use color_eyre::eyre::Result;

use crate::thread_pool::*;
use blackjack_shared::{
    card::Card,
    helpers::{rank_from_int, suit_from_int},
    web_socket::{WebSocketAction, WebSocketRequest},
};

use tungstenite::{accept, Message};

fn main() -> Result<()> {
    color_eyre::install()?;
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(8);

    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            println!("Incoming connection.");

            pool.execute(move || {
                let mut web_socket = accept(stream).unwrap();

                loop {
                    let msg = web_socket.read().unwrap();
                    let resp = handle_connection(msg);

                    resp.send_request(&mut web_socket);

                    if resp.action == WebSocketAction::Close {
                        web_socket.close(None).unwrap_or_else(|err| {
                            println!("{}", err);
                        });
                        break;
                    }
                }
            });
        } else {
            println!("Unsuccessful connection made.");
        }
    }

    drop(listener);
    Ok(())
}

fn handle_connection(msg: Message) -> WebSocketRequest {
    println!("Recieved message {}", msg);
    match msg {
        msg @ Message::Text(_) => {
            let text = msg.into_text().unwrap();
            WebSocketRequest {
                action: WebSocketAction::Send,
                message: text,
            }
        }
        Message::Close(_) => WebSocketRequest {
            action: WebSocketAction::Close,
            message: String::new(),
        },
        _ => WebSocketRequest {
            action: WebSocketAction::None,
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
