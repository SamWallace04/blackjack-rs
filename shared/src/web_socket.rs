use serde::{Deserialize, Serialize};

use crate::{card::Card, player::Player};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RegisterRequest {
    pub user_name: String,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RegisterResponse {
    pub url: String,
    pub is_host: bool,
    pub id: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum PublishTrigger {
    StartTurn {
        active_client_id: String,
        user_name: String,
        dealer_card: Option<Card>,
    },
    CardsDrawn {
        cards: Vec<Card>,
    },
    // TODO: Add the dealer into the results for the clients.
    RoundFinished(Vec<TurnResult>),
    GameFinished,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RequestCommand {
    Start,
    Bet(u32),
    DrawCards(u16),
    Hit,
    EndTurn(Player),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackjackRequest {
    pub command: RequestCommand,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub trigger: PublishTrigger,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub player: Player,
    pub end_state: EndState,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum EndState {
    Win,
    Loss,
    Blackjack,
    Push,
}
