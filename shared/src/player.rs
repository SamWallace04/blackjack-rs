use serde::{Deserialize, Serialize};

use crate::{
    card::{Card, Rank},
    helpers::get_rank_value,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PlayerType {
    Human,
    Dealer,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Player {
    pub user_name: String,
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

pub fn get_hand_value(hand: Vec<Card>) -> u32 {
    let mut values = vec![];

    (0..hand.len()).for_each(|i| {
        values.push(get_rank_value(hand[i].rank.clone()));
    });

    let mut value = values.into_iter().sum();

    if hand.into_iter().any(|c| c.rank == Rank::Ace) && value > 21 {
        value -= 10;
    }

    value
}
