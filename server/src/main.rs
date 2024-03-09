mod thread_pool;
mod web_socket;

use std::net::TcpListener;

use color_eyre::eyre::Result;

use crate::thread_pool::*;
use crate::web_socket::*;
use blackjack_shared::{
    card::Card,
    helpers::{rank_from_int, suit_from_int},
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

                    match resp.action {
                        WebSocketAction::Send => {
                            web_socket.send(Message::Text(resp.response)).unwrap();
                        }
                        WebSocketAction::Close => {
                            println!("Closing");
                            web_socket.close(None).unwrap_or_else(|error| {
                                println!("{}", error);
                            });
                            break;
                        }
                        WebSocketAction::None => {}
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

fn handle_connection(msg: Message) -> WebSocketResponse {
    println!("Recieved message {}", msg);
    match msg {
        msg @ Message::Text(_) => {
            let text = msg.into_text().unwrap();
            WebSocketResponse {
                action: WebSocketAction::Send,
                response: text,
            }
        }
        Message::Close(_) => WebSocketResponse {
            action: WebSocketAction::Close,
            response: String::new(),
        },
        _ => WebSocketResponse {
            action: WebSocketAction::None,
            response: String::new(),
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
