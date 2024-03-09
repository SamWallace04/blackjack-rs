use crate::card::Card;

pub enum PlayerType {
    Human,
    Dealer,
}

pub struct Player {
    pub player_type: PlayerType,
    pub hand: Vec<Card>,
    pub hand_value: u32,
    pub chips: u32,
    pub current_bet: u32,
}

pub enum PlayerAction {
    Hit,
    Stand,
    Double,
}
