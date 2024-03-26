use blackjack_shared::player::Player;
use tokio::sync::mpsc;
use warp::filters::ws::Message;

#[derive(Debug, Clone)]
pub struct Client {
    pub id: String,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
    pub position: usize,
    pub player: Player,
}
