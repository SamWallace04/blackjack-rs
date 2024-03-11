use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use uuid::Uuid;
use warp::{
    reject::Rejection,
    reply::{json, Reply},
    ws::{Message, WebSocket},
};

use crate::{client::Client, Clients};
use blackjack_shared::web_socket::*;

pub async fn register_handler(
    body: RegisterRequest,
    clients: Clients,
) -> Result<impl Reply, Rejection> {
    print!("Registration from: {}", body.user_id);
    let user_id = body.user_id;
    let uuid = Uuid::new_v4().simple().to_string();

    register_client(uuid.clone(), user_id, clients).await;
    Ok(json(&RegisterResponse {
        url: format!("ws://127.0.0.1:8000/ws/{}", uuid),
    }))
}

async fn register_client(id: String, user_id: usize, clients: Clients) {
    clients.lock().await.insert(
        id,
        Client {
            user_id,
            topics: vec![String::from("cats")],
            sender: None,
        },
    );
}

pub async fn unregister_handler(id: String, clients: Clients) -> Result<impl Reply, Rejection> {
    clients.lock().await.remove(&id);
    Ok(warp::http::StatusCode::OK)
}

pub async fn ws_handler(
    ws: warp::ws::Ws,
    id: String,
    clients: Clients,
) -> Result<impl Reply, Rejection> {
    let client = clients.lock().await.get(&id).cloned();
    match client {
        Some(c) => Ok(ws.on_upgrade(move |socket| client_connection(socket, id, clients, c))),
        None => Err(warp::reject::not_found()),
    }
}

pub async fn client_connection(ws: WebSocket, id: String, clients: Clients, mut client: Client) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();

    let client_rcv = UnboundedReceiverStream::new(client_rcv);
    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            eprintln!("error sending websocket msg: {}", e);
        }
    }));

    client.sender = Some(client_sender);
    clients.lock().await.insert(id.clone(), client.clone());

    println!("{} connected", id);

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                eprintln!("error receiving ws message for id: {}): {}", id.clone(), e);
                break;
            }
        };
        handle_client_msg(&id, msg, &clients, client.clone()).await;
    }

    clients.lock().await.remove(&id);
    println!("{} disconnected", id);
}

async fn handle_client_msg(id: &str, msg: Message, clients: &Clients, client: Client) {
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

    //TODO: Handle req.
    if let Some(sender) = &client.sender {
        let resp = BlackjackRequest {
            action: RequestAction::Send,
            command: RequestCommand::Info,
            message: req.message,
        };

        let _ = sender.send(Ok(Message::text(serde_json::to_string(&resp).unwrap())));
    }

    // let mut locked = clients.lock().await;
    // match locked.get_mut(id) {
    //     Some(v) => {
    //         v.topics = topics_req.topics;
    //     }
    //     None => return,
    // };
}
