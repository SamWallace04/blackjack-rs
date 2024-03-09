#[derive(Debug)]
pub enum WebSocketAction {
    Send,
    Close,
    None,
}

#[derive(Debug)]
pub struct WebSocketResponse {
    pub action: WebSocketAction,
    pub response: String,
}
