use blackjack_shared::player::*;
use blackjack_shared::web_socket::*;

use crate::card::draw_cards;
use crate::client::Client;
use crate::Clients;
use crate::Dealer;

/// Starts a new turn for the game.
pub async fn start_turn(client: &Client, dealer: &Dealer) -> PublishRequest {
    let dealer_cards = draw_cards(2);
    let mut dealer_lock = dealer.lock().await;

    dealer_lock.hand = dealer_cards.clone();
    dealer_lock.hand_value = get_hand_value(dealer_cards.clone());
    drop(dealer_lock);

    PublishRequest {
        trigger: PublishTrigger::StartTurn {
            active_client_id: client.id.clone(),
            user_name: client.player.user_name.clone(),
            dealer_card: Some(dealer.lock().await.hand[0].clone()),
        },
    }
}

/// Sets the player's bet amount using the given id of the client.
pub async fn bet(clients: &Clients, id: &str, amount: u32) {
    let mut clients_lock = clients.lock().await;
    let client_mut = clients_lock.iter_mut().find(|c| c.id == id).unwrap();
    client_mut.player.current_bet = amount;
}

pub async fn draw_cards_for_publish(n: u16, clients: &Clients, client: &Client) -> PublishRequest {
    let mut clients_lock = clients.lock().await;
    let client_mut = clients_lock.iter_mut().find(|c| c.id == client.id).unwrap();

    let drawn_cards = draw_cards(n);
    client_mut.player.hand.extend(drawn_cards.clone());
    client_mut.player.hand_value = get_hand_value(client_mut.player.hand.clone());

    drop(clients_lock);

    PublishRequest {
        trigger: PublishTrigger::CardsDrawn {
            cards: drawn_cards.clone(),
        },
    }
}

/// Ends the game turn. This will calcualte the end state for each player.
/// It will also check of all players have run out of chips and return a bool indicating if the
/// game should continue.
pub async fn end_turn(
    client: &mut Client,
    clients: &Clients,
    player: &Player,
    dealer: &Dealer,
) -> (PublishRequest, bool) {
    client.player = player.clone();

    let lock = clients.lock().await;
    let next_client = lock
        .iter()
        // TODO: This will break if someone leaves. Probably ok for this.
        .find(|c| c.position == client.position + 1);

    if let Some(c) = next_client {
        return (
            PublishRequest {
                trigger: PublishTrigger::StartTurn {
                    active_client_id: c.id.clone(),
                    user_name: c.player.user_name.clone(),
                    dealer_card: Some(dealer.lock().await.hand[0].clone()),
                },
            },
            true,
        );
    } else {
        drop(lock);
        // If we can't find any more clients then all players have finished.
        println!("No next client found, ending round.");

        // Play the dealer's turn.
        // TODO: Broadcase the dealer's turn to all clients.
        take_dealers_turn(&dealer).await;

        // End the game.
        let mut results = vec![];
        let mut clients_lock = clients.lock().await;
        let mut continue_playing = false;

        results.push(TurnResult {
            player: dealer.lock().await.clone(),
            end_state: EndState::Push, // Result for dealer is irrelevent.
        });

        for c in clients_lock.iter_mut() {
            // Calculate the end state for each player.
            println!("Calculating end state for {}", c.id);
            let end_state = calculate_end_state(&c.player, &dealer).await;

            handle_end_state(&mut c.player, end_state.clone());

            c.player.hand = vec![];
            c.player.hand_value = 0;

            results.push(TurnResult {
                player: c.player.clone(),
                end_state: end_state.clone(),
            });

            println!("Result: {:?}", end_state);
            // Keep playing until everyone is out of chips.
            if c.player.chips > 0 {
                continue_playing = true;
            }
        }

        drop(clients_lock);

        let pub_req = PublishRequest {
            trigger: PublishTrigger::RoundFinished(results),
        };

        (pub_req, continue_playing)
    }
}

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
