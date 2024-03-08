use std::fmt::{self};

use rand::seq::SliceRandom;
use rand::thread_rng;

#[derive(Clone)]
pub enum Suit {
    Spades,
    Hearts,
    Diamonds,
    Clubs,
}

impl Suit {
    pub fn from_int(a: u32) -> Suit {
        match a % 4 {
            0 => Suit::Spades,
            1 => Suit::Hearts,
            2 => Suit::Diamonds,
            _ => Suit::Clubs,
        }
    }
}

impl fmt::Display for Suit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Suit::Spades => "♠",
                Suit::Hearts => "♥",
                Suit::Diamonds => "♦",
                Suit::Clubs => "♣",
            }
        )
    }
}

#[derive(Clone, PartialEq)]
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

impl Rank {
    pub fn from_int(a: u32) -> Rank {
        match a % 13 {
            0 => Rank::Two,
            1 => Rank::Three,
            2 => Rank::Four,
            3 => Rank::Five,
            4 => Rank::Six,
            5 => Rank::Seven,
            6 => Rank::Eight,
            7 => Rank::Nine,
            8 => Rank::Ten,
            9 => Rank::Jack,
            10 => Rank::Queen,
            11 => Rank::King,
            _ => Rank::Ace,
        }
    }

    pub fn get_value(&self) -> u32 {
        match self {
            Rank::Two => 2,
            Rank::Three => 3,
            Rank::Four => 4,
            Rank::Five => 5,
            Rank::Six => 6,
            Rank::Seven => 7,
            Rank::Eight => 8,
            Rank::Nine => 9,
            Rank::Ten => 10,
            Rank::Jack => 10,
            Rank::Queen => 10,
            Rank::King => 10,
            Rank::Ace => 11, // The ace's value needs to be calculated in the context of a hand.
        }
    }
}

impl fmt::Display for Rank {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Rank::Two => "2",
                Rank::Three => "3",
                Rank::Four => "4",
                Rank::Five => "5",
                Rank::Six => "6",
                Rank::Seven => "7",
                Rank::Eight => "8",
                Rank::Nine => "9",
                Rank::Ten => "T",
                Rank::Jack => "J",
                Rank::Queen => "Q",
                Rank::King => "K",
                Rank::Ace => "A",
            }
        )
    }
}

#[derive(Clone)]
pub struct Card {
    pub suit: Suit,
    pub rank: Rank,
}

impl fmt::Display for Card {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.rank, self.suit)
        /*)writedoc!(
            f,
            "
               /-----\\
               |{}    |
               |  {}  |
               |    {}|
               \\-----/
               ",
            self.rank,
            self.suit,
            self.rank
        )*/
    }
}

pub fn draw_cards(deck: &mut Vec<Card>, num_to_draw: u16) -> Vec<Card> {
    let mut cards: Vec<Card> = Vec::new();
    let mut num = num_to_draw;

    while num > 0 {
        cards.push(match deck.pop() {
            Some(card) => card,
            None => {
                let new_deck = create_playing_deck(1);
                deck.extend(new_deck);
                deck.pop().unwrap()
            }
        });
        num -= 1;
    }
    cards
}

pub fn create_playing_deck(num_of_decks: u8) -> Vec<Card> {
    let mut deck: Vec<Card> = (0..num_of_decks)
        .flat_map(|_d| {
            (0..4).flat_map(|s| {
                (0..13).map(move |r| Card {
                    rank: Rank::from_int(r),
                    suit: Suit::from_int(s),
                })
            })
        })
        .collect();

    deck.shuffle(&mut thread_rng());
    deck
}
