use std::{io::Read, io::Write};

use blackjack_shared::player::{get_hand_value, PlayerAction};
use blackjack_shared::web_socket::*;
use blackjack_shared::{card::Card, player::Player};
use tungstenite::WebSocket;

use crate::*;

pub fn start_turn<S>(socket: &mut WebSocket<S>, me: &mut Player, dealer_card: Card)
where
    S: Read + Write,
{
    let draw_req = BlackjackRequest {
        command: RequestCommand::DrawCards(2),
    };

    me.current_bet = bet(me.chips);

    send_request(
        BlackjackRequest {
            command: RequestCommand::Bet(me.current_bet),
        },
        socket,
    );

    let draw_res = send_request_and_wait_for_response(draw_req, socket);

    let drawn_cards = handle_draw_cards(draw_res.as_str());

    me.hand = drawn_cards.clone();

    print!("You drew the following card(s): ");
    print_cards_in_hand(drawn_cards.clone(), None);
    println!();

    println!("Your hand value is: {}", get_hand_value(me.hand.clone()));

    println!("The dealer's face card is: {}", dealer_card);

    play_turn(me, socket);

    println!("Your turn has ended.");

    let end_turn_req = BlackjackRequest {
        command: RequestCommand::EndTurn(me.clone()),
    };

    send_request(end_turn_req, socket);
}

fn play_turn<S>(player: &mut Player, socket: &mut WebSocket<S>)
where
    S: Read + Write,
{
    let mut can_take_action = true;

    while can_take_action {
        println!("\nWhat action would you like to take? (Hit, Stand, Double or Split)");
        let action = get_player_action();

        match action {
            PlayerAction::Hit => {
                hit(player, socket);
            }
            PlayerAction::Stand => {
                can_take_action = false;
            }
            PlayerAction::Double => {
                if player.current_bet * 2 > player.chips {
                    println!("You don't have enough chips to double your bet!");
                    continue;
                }

                hit(player, socket);

                player.current_bet *= 2;
                send_request(
                    BlackjackRequest {
                        command: RequestCommand::Bet(player.current_bet),
                    },
                    socket,
                );

                can_take_action = false;
            }
        }
        println!("Your hand is now: ");
        print_cards_in_hand(player.hand.clone(), None);
        println!();

        player.hand_value = get_hand_value(player.hand.clone());
        println!("Your hand value is: {}", player.hand_value);

        if player.hand_value > 21 {
            println!("You busted!");
            can_take_action = false;
        }
    }
}

fn hit(player: &mut Player, socket: &mut WebSocket<impl Read + Write>) {
    let req = BlackjackRequest {
        command: RequestCommand::Hit,
    };

    // TODO: Might want the card to be returned in the message.
    send_request(req, socket);

    let res = wait_for_message(socket);

    let cards_drawn = handle_draw_cards(res.into_text().unwrap().as_str());

    print!("You drew the following card(s): ");
    print_cards_in_hand(cards_drawn.clone(), None);
    println!();

    player.hand.extend(cards_drawn);
}

pub fn handle_bets(player: &Player, end_state: &EndState, is_current_player: bool) {
    let name = if is_current_player {
        "You"
    } else {
        &player.user_name
    };
    match end_state {
        EndState::Win => {
            println!("{} won! Amount paid out: {}", name, player.current_bet);
        }
        EndState::Loss => {
            println!("{} lost! Chips lost: {}", name, player.current_bet);
        }
        EndState::Blackjack => {
            println!(
                "{} got a blackjack! Amount paid out: {}",
                name,
                player.current_bet * 3
            );
        }
        EndState::Push => println!("{} and the dealer drew. Nothing paid out.", name), // Nothing to do on a push
    }
}

fn handle_draw_cards(msg: &str) -> Vec<Card> {
    let mut drawn_cards: Vec<Card> = vec![];

    if let Ok(res) = serde_json::from_str::<PublishRequest>(msg) {
        if let PublishTrigger::CardsDrawn { cards } = res.trigger {
            drawn_cards = cards;
        }
    }

    drawn_cards
}

pub fn print_cards_in_hand(hand: Vec<Card>, num_to_show: Option<usize>) {
    let num = num_to_show.unwrap_or(hand.len());
    (0..num).for_each(|n| print!("{} ", hand[n]));
}
