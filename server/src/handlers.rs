use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    reject::Rejection,
    reply::{json, Reply},
    ws::{Message, WebSocket},
};

use crate::{card::draw_cards, client::Client, Clients};
use blackjack_shared::{card::Card, web_socket::*};

pub async fn register_handler(
    body: RegisterRequest,
    clients: Clients,
) -> Result<impl Reply, Rejection> {
    println!("Registration from: {}", body.user_name);
    let user_name = body.user_name;
    let uuid = Uuid::new_v4().simple().to_string();

    let is_host = clients.lock().await.len() == 0;

    register_client(uuid.clone(), user_name, clients).await;
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
        is_host,
        id: uuid,
    }))
}

async fn register_client(id: String, user_name: String, clients: Clients) {
    let position = clients.lock().await.len();
    println!("position: {}", position);
    clients.lock().await.push(Client {
        id,
        user_name,
        sender: None,
        position,
    });
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply, Rejection> {
    clients.lock().await.retain(|c| c.id != id);
    Ok(warp::http::StatusCode::OK)
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    id: String,
    clients: Clients,
) -> Result<impl Reply, Rejection> {
    let lock = clients.lock().await.clone();
    let client = lock.iter().find(|c| c.id == id).clone();
    match client {
        Some(_) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    // Limit the scope of the lock and mut reference.
    {
        // Update the sender of the new client in place.
        let mut lock = clients.lock().await;
        let client = lock.iter_mut().find(|c| c.id == id);

        if let Some(c) = client {
            c.sender = Some(client_sender);
        }
    }

    println!("{} connected", id);

    let client = clients
        .lock()
        .await
        .iter()
        .find(|c| c.id == id)
        .unwrap()
        .clone();

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        handle_client_msg(&id, msg, clients.clone(), client.clone()).await;
    }

    clients.lock().await.retain(|c| c.id != id);
    println!("{} disconnected", id);
}

pub async fn publish_handler(
    body: PublishRequest,
    clients: Clients,
) -> Result<impl Reply, Rejection> {
    publish(body, clients, None).await
}

async fn publish(
    body: PublishRequest,
    clients: Clients,
    filter_client_id: Option<String>,
) -> Result<impl Reply, Rejection> {
    println!(
        "Attempting to publish message {}",
        serde_json::to_string(&body).unwrap()
    );
    clients.lock().await.iter_mut().for_each(|client| {
        if let Some(filter) = &filter_client_id {
            if client.id == *filter {
                return;
            }
        }

        if let Some(sender) = &client.sender {
            println!("Sending message to {}", client.user_name);
            let _ = sender.send(Ok(Message::text(serde_json::to_string(&body).unwrap())));
        }
    });

    println!("Message published");
    Ok(warp::http::StatusCode::OK)
}

async fn handle_client_msg(id: &str, msg: Message, clients: Clients, client: Client) {
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
        return;
    }

    let req: BlackjackRequest = match serde_json::from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to request: {}", e);
            return;
        }
    };

    //TODO: Split out each arm into a function.
    if let Some(sender) = &client.sender {
        match req.command {
            RequestCommand::Start => {
                let pub_req = PublishRequest {
                    // TODO: Move the id value into the trigger enum.
                    trigger: PublishTrigger::StartTurn {
                        active_client_id: client.id,
                        user_name: client.user_name.clone(),
                    },
                };
                let _ = publish(pub_req, clients, None).await;
                //TODO: Start the game.
            }
            RequestCommand::EndTurn => {
                let lock = clients.lock().await;
                let next_client = lock
                    .iter()
                    // TODO: This will break if someone leaves. Probably ok for this.
                    .find(|c| c.position == client.position + 1)
                    .clone();

                if let Some(c) = next_client {
                    let req = PublishRequest {
                        trigger: PublishTrigger::StartTurn {
                            active_client_id: c.id.clone(),
                            user_name: c.user_name.clone(),
                        },
                    };
                    // Make sure the lock gets dropped before calling the publish handler.
                    drop(lock);
                    let _ = publish(req, clients, None).await;
                } else {
                    println!("No next client found");
                }
            }
            RequestCommand::DrawCards(n) => {
                draw_cards_and_publish(n, client.clone(), clients.clone()).await;
            }
            RequestCommand::Hit => {
                draw_cards_and_publish(1, client.clone(), clients.clone()).await;
            }
            _ => {}
        };
        let resp = BlackjackRequest {
            command: RequestCommand::Info,
            message: req.message,
        };

        let _ = sender.send(Ok(Message::text(serde_json::to_string(&resp).unwrap())));
    }

    async fn draw_cards_and_publish(n: u16, client: Client, clients: Clients) -> Vec<Card> {
        let drawn_cards = draw_cards(n);
        let pub_req = PublishRequest {
            trigger: PublishTrigger::CardsDrawn {
                cards: drawn_cards.clone(),
            },
        };
        let _ = publish(pub_req, clients, None).await;

        drawn_cards
    }

    // let mut locked = clients.lock().await;
    // match locked.get_mut(id) {
    //     Some(v) => {
    //         v.topics = topics_req.topics;
    //     }
    //     None => return,
    // };
}
