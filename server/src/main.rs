mod card;
mod client;
mod game;
mod handlers;

use std::{convert::Infallible, sync::Arc};

use blackjack_shared::player::{Player, PlayerType};
use color_eyre::eyre::*;
use tokio::sync::Mutex;
use warp::Filter;

use crate::client::Client;

type Clients = Arc<Mutex<Vec<Client>>>;
type Dealer = Arc<Mutex<Player>>;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    let clients: Clients = Arc::new(Mutex::new(vec![]));
    let dealer: Dealer = Arc::new(Mutex::new(Player {
        user_name: "Dealer".to_string(),
        player_type: PlayerType::Dealer,
        hand: vec![],
        hand_value: 0,
        chips: 0,
        current_bet: 0,
    }));

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
        .and(with_dealer(dealer.clone()))
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

fn with_dealer(dealer: Dealer) -> impl Filter<Extract = (Dealer,), Error = Infallible> + Clone {
    warp::any().map(move || dealer.clone())
}
