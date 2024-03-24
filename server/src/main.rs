mod card;
mod client;
mod handlers;
mod thread_pool;

use std::{convert::Infallible, sync::Arc};

use color_eyre::eyre::*;
use tokio::sync::Mutex;
use warp::Filter;

use crate::client::Client;

type Clients = Arc<Mutex<Vec<Client>>>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let clients: Clients = Arc::new(Mutex::new(vec![]));

    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handlers::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handlers::unregister_handler));

    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handlers::publish_handler);

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handlers::ws_handler);

    let routes = ws_route
        .or(publish)
        .or(register_routes)
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;

    Ok(())
}

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

// fn handle_message(msg: Message) -> WebSocketRequest {
//     println!("Recieved message {}", msg);
//     match msg {
//         msg @ Message::Text(_) => {
//             let text = msg.into_text().unwrap();
//             let request: WebSocketRequest =
//                 serde_json::from_str(&text).expect("Request to be of type WebSocketRequest");
//
//             match request.command {
//                 //TODO: Return the player object.
//                 RequestCommand::Start => WebSocketRequest {
//                     action: WebSocketAction::Send,
//                     command: RequestCommand::Info,
//                     message: request.message,
//                 },
//                 _ => panic!(),
//             }
//         }
//         Message::Close(_) => WebSocketRequest {
//             action: WebSocketAction::Close,
//             command: RequestCommand::Info,
//             message: String::new(),
//         },
//         _ => WebSocketRequest {
//             action: WebSocketAction::None,
//             command: RequestCommand::Info,
//             message: String::new(),
//         },
//     }
// }

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
