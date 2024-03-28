use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    reject::Rejection,
    reply::{json, Reply},
    ws::{Message, WebSocket},
};

use crate::{client::Client, game::*, Clients, Dealer};
use blackjack_shared::{
    player::{Player, PlayerType},
    web_socket::*,
};

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
    clients.lock().await.push(Client {
        id,
        sender: None,
        position,
        player: Player {
            user_name,
            player_type: PlayerType::Human,
            hand: vec![],
            hand_value: 0,
            chips: 500,
            current_bet: 0,
        },
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
    dealer: Dealer,
) -> Result<impl Reply, Rejection> {
    let lock = clients.lock().await.clone();
    let client = lock.iter().find(|c| c.id == id);
    match client {
        Some(_) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, dealer))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, dealer: Dealer) {
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
        handle_client_msg(&id, msg, clients.clone(), client.clone(), dealer.clone()).await;
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
            println!("Sending message to {}", client.player.user_name);
            let _ = sender.send(Ok(Message::text(serde_json::to_string(&body).unwrap())));
        }
    });

    println!("Message published");
    Ok(warp::http::StatusCode::OK)
}

async fn handle_client_msg(
    id: &str,
    msg: Message,
    clients: Clients,
    mut client: Client,
    dealer: Dealer,
) {
    // TODO: Try and limit cloning - change functions to borrow where possible.
    // TODO: Add testing.
    println!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    if message == "ping" || message == "ping\n" {
        return;
    }

    let req: BlackjackRequest = match serde_json::from_str(message) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error while parsing message to request: {}", e);
            return;
        }
    };

    if let Some(_sender) = &client.sender {
        match req.command {
            RequestCommand::Start => {
                let pub_req = start_turn(&client, &dealer).await;
                let _ = publish(pub_req, clients, None).await;
            }
            RequestCommand::Bet(amount) => {
                bet(&clients, &client.id, amount).await;
                // TODO: Publish the bet amount to all clients.
            }
            RequestCommand::EndTurn(player) => {
                let (pub_req, continue_playing) =
                    end_turn(&mut client, &clients, &player, &dealer).await;

                let _ = publish(pub_req, clients.clone(), None).await;

                if !continue_playing {
                    let game_finished_req = PublishRequest {
                        trigger: PublishTrigger::GameFinished,
                    };

                    let _ = publish(game_finished_req, clients.clone(), None).await;
                }
            }
            RequestCommand::DrawCards(n) => {
                let pub_req = draw_cards_for_publish(n, &clients, &client).await;
                let _ = publish(pub_req, clients.clone(), None).await;
            }
            RequestCommand::Hit => {
                let pub_req = draw_cards_for_publish(1, &clients, &client).await;
                let _ = publish(pub_req, clients.clone(), None).await;
            }
        };
    }
}
