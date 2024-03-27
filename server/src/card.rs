use blackjack_shared::card::Card;
use blackjack_shared::helpers::{rank_from_int, suit_from_int};
use rand::seq::SliceRandom;
use rand::thread_rng;

pub fn draw_cards(num_to_draw: u16) -> Vec<Card> {
    // The state of the deck is somewhat irrelevant for blackjack games so create a new one each
    // time.
    let mut deck = create_playing_deck(1);
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
                    rank: rank_from_int(r),
                    suit: suit_from_int(s),
                })
            })
        })
        .collect();

    deck.shuffle(&mut thread_rng());
    deck
}
