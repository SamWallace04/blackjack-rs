use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Rank {
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
    Ace,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}
