use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use tungstenite::{Message, WebSocket};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RegisterRequest {
    pub user_id: usize,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct RegisterResponse {
    pub url: String,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum RequestAction {
    Send,
    Close,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RequestCommand {
    Start,
    StartTurn,
    Action,
    End,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlackjackRequest {
    pub action: RequestAction,
    pub command: RequestCommand,
    pub message: String,
}
