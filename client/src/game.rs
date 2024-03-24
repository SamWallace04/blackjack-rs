use std::{io::Read, io::Write};

use blackjack_shared::player::{get_hand_value, PlayerAction, PlayerType};
use blackjack_shared::web_socket::*;
use blackjack_shared::{card::Card, player::Player};
use tungstenite::WebSocket;

use crate::*;

pub fn start_turn<S>(socket: &mut WebSocket<S>)
where
    S: Read + Write,
{
    let draw_req = BlackjackRequest {
        command: RequestCommand::DrawCards(2),
        message: None,
    };

    let draw_res = send_request_and_wait_for_response(draw_req, socket);

    let drawn_cards = handle_draw_cards(draw_res.as_str());

    let mut me = Player {
        player_type: PlayerType::Human,
        hand: drawn_cards.clone(),
        hand_value: 0,
        chips: 500,
        current_bet: 0,
    };

    me.current_bet = bet(me.chips);

    print!("You drew the following card(s): ");
    print_cards_in_hand(drawn_cards.clone(), None);
    println!("");

    play_turn(&mut me, socket);

    send_request(
        BlackjackRequest {
            command: RequestCommand::EndTurn,
            message: None,
        },
        socket,
    );
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
                stand(socket);
                can_take_action = false;
            }
            PlayerAction::Double => {
                if player.current_bet * 2 > player.chips {
                    println!("You don't have enough chips to double your bet!");
                    continue;
                }

                double(player, socket);
                can_take_action = false;
            }
        }
        println!("Your hand is now: ");
        print_cards_in_hand(player.hand.clone(), None);
        println!("");

        player.hand_value = get_hand_value(player.hand.clone());
        println!("Your hand value is: {}", player.hand_value);
    }
}

fn hit(player: &mut Player, socket: &mut WebSocket<impl Read + Write>) {
    let req = BlackjackRequest {
        command: RequestCommand::Hit,
        message: None,
    };

    // TODO: Might want the card to be returned in the message.
    send_request_and_discard_response(req, socket);

    let res = wait_for_message(socket);

    let cards_drawn = handle_draw_cards(res.into_text().unwrap().as_str());

    print!("You drew the following card(s): ");
    print_cards_in_hand(cards_drawn.clone(), None);
    println!("");

    player.hand.extend(cards_drawn);
}

fn stand(socket: &mut WebSocket<impl Read + Write>) {
    let req = BlackjackRequest {
        command: RequestCommand::Stand,
        message: None,
    };

    println!("Standing");
    send_request_and_discard_response(req, socket);
}

fn double(player: &mut Player, socket: &mut WebSocket<impl Read + Write>) {
    let req = BlackjackRequest {
        command: RequestCommand::Double,
        message: None,
    };

    println!("Doubling");
    send_request_and_discard_response(req, socket);

    let res = wait_for_message(socket);

    let cards_drawn = handle_draw_cards(res.into_text().unwrap().as_str());

    print!("You drew the following card(s): ");
    print_cards_in_hand(cards_drawn.clone(), None);
    println!("");

    player.hand.extend(cards_drawn);
}

fn handle_draw_cards(msg: &str) -> Vec<Card> {
    let mut drawn_cards: Vec<Card> = vec![];

    if let Ok(res) = serde_json::from_str::<PublishRequest>(&msg) {
        match res.trigger {
            PublishTrigger::CardsDrawn { cards } => {
                drawn_cards = cards;
            }
            _ => {}
        }
    }

    drawn_cards
}

pub fn print_cards_in_hand(hand: Vec<Card>, num_to_show: Option<usize>) {
    let num = num_to_show.unwrap_or(hand.len());
    for n in 0..num {
        print!("{} ", hand[n])
    }
}
