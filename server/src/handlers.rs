use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    reject::Rejection,
    reply::{json, Reply},
    ws::{Message, WebSocket},
};

use crate::{
    card::draw_cards,
    client::Client,
    game::{calculate_end_state, handle_end_state, take_dealers_turn},
    Clients, Dealer,
};
use blackjack_shared::{
    card::Card,
    player::{get_hand_value, Player, PlayerType},
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

    //TODO: Split out each arm into a function.
    if let Some(_sender) = &client.sender {
        match req.command {
            RequestCommand::Start => {
                let dealer_cards = draw_cards(2);
                let mut dealer_lock = dealer.lock().await;

                dealer_lock.hand = dealer_cards.clone();
                dealer_lock.hand_value = get_hand_value(dealer_cards.clone());
                drop(dealer_lock);

                let pub_req = PublishRequest {
                    trigger: PublishTrigger::StartTurn {
                        active_client_id: client.id.clone(),
                        user_name: client.player.user_name.clone(),
                        dealer_card: Some(dealer.lock().await.hand[0].clone()),
                    },
                };

                let _ = publish(pub_req, clients, None).await;
            }
            RequestCommand::Bet(amount) => {
                let mut clients_lock = clients.lock().await;
                let client_mut = clients_lock.iter_mut().find(|c| c.id == id).unwrap();
                client_mut.player.current_bet = amount;
                // TODO: Publish the bet amount to all clients.
            }
            RequestCommand::EndTurn(player) => {
                client.player = player.clone();

                let lock = clients.lock().await;
                let next_client = lock
                    .iter()
                    // TODO: This will break if someone leaves. Probably ok for this.
                    .find(|c| c.position == client.position + 1);

                if let Some(c) = next_client {
                    let req = PublishRequest {
                        trigger: PublishTrigger::StartTurn {
                            active_client_id: c.id.clone(),
                            user_name: c.player.user_name.clone(),
                            dealer_card: Some(dealer.lock().await.hand[0].clone()),
                        },
                    };
                    // Make sure the lock gets dropped before calling the publish handler.
                    drop(lock);
                    let _ = publish(req, clients, None).await;
                } else {
                    drop(lock);
                    // If we can't find any more clients then all players have finished.
                    println!("No next client found, ending round.");

                    // Play the dealer's turn.
                    // TODO: Broadcase the dealer's turn to all clients.
                    take_dealers_turn(&dealer).await;

                    // End the game.
                    let mut results = vec![];
                    let mut clients_lock = clients.lock().await;
                    let mut continue_playing = false;

                    results.push(TurnResult {
                        player: dealer.lock().await.clone(),
                        end_state: EndState::Push, // Result for dealer is irrelevent.
                    });

                    for c in clients_lock.iter_mut() {
                        // Calculate the end state for each player.
                        println!("Calculating end state for {}", c.id);
                        let end_state = calculate_end_state(&c.player, &dealer).await;

                        handle_end_state(&mut c.player, end_state.clone());

                        c.player.hand = vec![];
                        c.player.hand_value = 0;

                        results.push(TurnResult {
                            player: c.player.clone(),
                            end_state: end_state.clone(),
                        });

                        println!("Result: {:?}", end_state);
                        // Keep playing until everyone is out of chips.
                        if c.player.chips > 0 {
                            continue_playing = true;
                        }
                    }

                    drop(clients_lock);

                    let pub_req = PublishRequest {
                        trigger: PublishTrigger::RoundFinished(results),
                    };

                    let _ = publish(pub_req, clients.clone(), None).await;

                    if !continue_playing {
                        let pub_req = PublishRequest {
                            trigger: PublishTrigger::GameFinished,
                        };

                        let _ = publish(pub_req, clients.clone(), None).await;
                    }
                }
            }
            RequestCommand::DrawCards(n) => {
                draw_cards_and_publish(n, clients.clone(), client).await;
            }
            RequestCommand::Hit => {
                draw_cards_and_publish(1, clients.clone(), client).await;
            }
        };
    }

    async fn draw_cards_and_publish(n: u16, clients: Clients, client: Client) -> Vec<Card> {
        let mut clients_lock = clients.lock().await;
        let client_mut = clients_lock.iter_mut().find(|c| c.id == client.id).unwrap();

        let drawn_cards = draw_cards(n);
        client_mut.player.hand.extend(drawn_cards.clone());
        client_mut.player.hand_value = get_hand_value(client_mut.player.hand.clone());

        drop(clients_lock);

        let pub_req = PublishRequest {
            trigger: PublishTrigger::CardsDrawn {
                cards: drawn_cards.clone(),
            },
        };
        let _ = publish(pub_req, clients, None).await;

        drawn_cards
    }
}
