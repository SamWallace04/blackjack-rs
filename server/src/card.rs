use rand::seq::SliceRandom;
use rand::thread_rng;

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
