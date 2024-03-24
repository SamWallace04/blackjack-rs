use serde::{Deserialize, Serialize};

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
    },
    CardDrawn,
    TurnEnded,
    GameFinished,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RequestCommand {
    Start,
    StartTurn,
    Action,
    EndTurn,
    Info,
    Close,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackjackRequest {
    pub command: RequestCommand,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishRequest {
    pub trigger: PublishTrigger,
}
