use crate::card::Card;

#[derive(Debug)]
pub enum PlayerType {
    Human,
    Dealer,
}

#[derive(Debug)]
pub struct Player {
    pub player_type: PlayerType,
    pub hand: Vec<Card>,
    pub hand_value: u32,
    pub chips: u32,
    pub current_bet: u32,
}

#[derive(Debug)]
pub enum PlayerAction {
    Hit,
    Stand,
    Double,
}
