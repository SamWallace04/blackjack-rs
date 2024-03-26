use blackjack_shared::player::*;
use blackjack_shared::web_socket::EndState;

use crate::card::draw_cards;
use crate::Dealer;

pub async fn take_dealers_turn(dealer_arc: &Dealer) {
    let mut dealer = dealer_arc.lock().await;
    dealer.hand_value = get_hand_value(dealer.hand.to_owned());

    // Aways stand on >= 17
    while dealer.hand_value < 17 {
        let card = draw_cards(1)[0].to_owned();
        dealer.hand.push(card);
        dealer.hand_value = get_hand_value(dealer.hand.to_owned());
    }

    println!("Dealer's hand: {:?}", dealer.hand);
}

pub async fn calculate_end_state(player: &Player, dealer: &Dealer) -> EndState {
    let player_value = player.hand_value;
    let dealer_value = dealer.lock().await.hand_value;

    if dealer_value > 21 && player_value > 21 || dealer_value == player_value {
        EndState::Push
    } else if is_blackjack(player) {
        EndState::Blackjack
    } else if dealer_value > 21 || (player_value <= 21 && player_value > dealer_value) {
        EndState::Win
    } else {
        EndState::Loss
    }
}

fn is_blackjack(player: &Player) -> bool {
    player.hand.len() == 2 && player.hand_value == 21
}

pub fn handle_end_state(player: &mut Player, end_state: EndState) {
    match end_state {
        EndState::Win => {
            player.chips += player.current_bet;
        }
        EndState::Loss => {
            player.chips -= player.current_bet;
        }
        EndState::Blackjack => {
            player.chips += player.current_bet * 3;
        }
        EndState::Push => {} // Nothing to do on a push
    }
}
