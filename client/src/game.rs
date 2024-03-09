use crate::card::*;
use crate::player::*;

pub fn start_game() {
    let mut deck = create_playing_deck(4);

    let mut dealer = Player {
        player_type: PlayerType::Dealer,
        hand: draw_cards(&mut deck, 2),
        hand_value: 0,
        chips: 0,
        current_bet: 0,
    };

    let me = Player {
        player_type: PlayerType::Human,
        hand: draw_cards(&mut deck, 2),
        hand_value: 0,
        chips: 500,
        current_bet: 0,
    };

    let mut players = vec![me];

    loop {
        match players.iter().find(|p| p.chips > 0) {
            Some(_i) => start_round(&mut deck, &mut players, &mut dealer),
            None => break,
        }
    }

    println!("You are all out of chips, game over!")
}

fn start_round(deck: &mut Vec<Card>, players: &mut Vec<Player>, dealer: &mut Player) {
    for n in 0..players.len() {
        players[n].current_bet = bet(players[n].chips);
        // Technically out of sequence but it doesn't really matter.
        players[n].hand = draw_cards(deck, 2)
    }

    dealer.hand = draw_cards(deck, 2);

    for n in 0..players.len() {
        println!("Your hand is: ");
        print_cards_in_hand(players[n].hand.to_vec(), 2);
        players[n].hand_value = get_hand_value(players[n].hand.to_owned());
        println!("\n{}", players[n].hand_value)
    }

    println!("\nThe dealer's face card is: ");
    print_cards_in_hand(dealer.hand.to_vec(), 1);

    play_round(deck, players, dealer)
}

fn play_round(deck: &mut Vec<Card>, players: &mut Vec<Player>, dealer: &mut Player) {
    for i in 0..players.len() {
        let mut can_take_action = true;

        while can_take_action {
            println!("\nWhat action would you like to take? (Hit, Stand, Double or Split)");
            let action = get_player_action();

            match action {
                PlayerAction::Hit => {
                    let drawn_card = draw_cards(deck, 1)[0].to_owned();
                    println!("You drew: ");
                    println!("{}", drawn_card.to_string());
                    players[i].hand.push(drawn_card);
                }
                PlayerAction::Stand => can_take_action = false,
                PlayerAction::Double => {
                    if players[i].current_bet * 2 > players[i].chips {
                        println!("You don't have the chips to double up.");
                        continue;
                    }

                    let drawn_card = draw_cards(deck, 1)[0].to_owned();
                    println!("You drew: ");
                    println!("{}", drawn_card.to_string());
                    players[i].hand.push(drawn_card);
                    players[i].current_bet *= 2;
                    can_take_action = false;
                }
            }

            players[i].hand_value = get_hand_value(players[i].hand.to_owned());

            println!("Your total hand value is: {}", players[i].hand_value);

            if players[i].hand_value > 21 {
                println!("You have bust! Better luck next time.");
                can_take_action = false
            }
        }
    }

    take_dealers_turn(deck, dealer);

    println!("The dealer's hand is: ");
    print_cards_in_hand(dealer.hand.to_vec(), 0);

    println!("\nThe dealer's score is {}", dealer.hand_value);

    calculate_end_state(players, dealer)
}

fn calculate_end_state(players: &mut Vec<Player>, dealer: &mut Player) {
    for i in 0..players.len() {
        let player = &players[i];
        let player_value = players[i].hand_value;
        let dealer_value = dealer.hand_value;
        let end_state: EndState;

        if dealer_value > 21 && player_value > 21 || dealer.hand_value == player_value {
            end_state = EndState::Push
        } else if is_blackjack(&player) {
            println!(
                "Wow you got blackjack! You won {} chips.",
                player.current_bet * 3
            );
            end_state = EndState::Blackjack;
        } else if dealer_value > 21 || (player_value <= 21 && player_value > dealer_value) {
            println!("Congrats, you won {} chips.", player.current_bet);
            end_state = EndState::Win;
        } else {
            println!("Unlucky, you lost {} chips.", player.current_bet);
            end_state = EndState::Loss;
        }

        handle_bets(&mut players[i], end_state);
    }
}

fn is_blackjack(player: &Player) -> bool {
    player.hand.len() == 2 && player.hand_value == 21
}

fn handle_bets(player: &mut Player, end_state: EndState) {
    match end_state {
        EndState::Win => player.chips += player.current_bet,
        EndState::Loss => player.chips -= player.current_bet,
        EndState::Blackjack => player.chips += player.current_bet * 3,
        EndState::Push => println!("You and the dealer drew. Nothing paid out."), // Nothing to do on a push
    }
}

fn take_dealers_turn(deck: &mut Vec<Card>, dealer: &mut Player) {
    dealer.hand_value = get_hand_value(dealer.hand.to_owned());

    // Aways stand on >= 17
    while dealer.hand_value < 17 {
        let card = draw_cards(deck, 1)[0].to_owned();
        dealer.hand.push(card);
        dealer.hand_value = get_hand_value(dealer.hand.to_owned());
    }
}

fn get_hand_value(hand: Vec<Card>) -> u32 {
    let mut values = vec![];

    for i in 0..hand.len() {
        values.push(hand[i].rank.get_value());
    }

    let mut value = values.into_iter().sum();

    if hand.into_iter().any(|c| c.rank == Rank::Ace) {
        if value > 21 {
            value -= 10;
        }
    }

    value
}

fn print_cards_in_hand(hand: Vec<Card>, mut num_to_show: usize) {
    if num_to_show == 0 {
        num_to_show = hand.len();
    }
    for n in 0..num_to_show {
        print!("{}", hand[n])
    }
}

enum EndState {
    Win,
    Loss,
    Blackjack,
    Push,
}
