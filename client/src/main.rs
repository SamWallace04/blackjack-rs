mod game;
mod player_input;
mod web_socket;

use crate::{game::*, player_input::*, web_socket::*};

use blackjack_shared::{
    player::{Player, PlayerType},
    web_socket::*,
};
use color_eyre::eyre::Result;
use tungstenite::connect;
use url::Url;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;

    println!("Please enter your username:");
    let my_user_name = get_user_input();

    let http_client = reqwest::Client::new();
    let res_json = http_client
        .post("http://127.0.0.1:8000/register")
        .json(&RegisterRequest {
            user_name: my_user_name.to_string(),
        })
        .send()
        .await?
        .text()
        .await?;

    let res: RegisterResponse = serde_json::from_str(res_json.as_str())?;

    let (mut socket, _) = connect(Url::parse(&res.url).unwrap()).expect("Can't connect");

    println!("Connected to the server");
    let client_id = res.id.clone();

    if res.is_host {
        loop {
            println!("You are the host. Enter 'start' to begin the game.");
            let input = get_user_input();
            if input == "start" {
                let req = BlackjackRequest {
                    command: RequestCommand::Start,
                };
                send_request(req, &mut socket);
                break;
            }
        }
    } else {
        println!("Waiting for the host to start the game...");
        wait_for_message(&mut socket);
    }

    println!("The game is starting.");

    let mut me = Player {
        user_name: my_user_name.clone(),
        player_type: PlayerType::Human,
        hand: vec![],
        hand_value: 0,
        chips: 500,
        current_bet: 0,
    };

    //Check the start response. If the username matches ours then start the game.
    //Otherwise wait in a loop checking messages and updating the output as required.
    //Once a start message with our name has been send start playing the game.
    let mut current_player_name = String::new();
    loop {
        // TODO: Better handle none publish messages.
        let message = wait_for_message(&mut socket);
        let request: PublishRequest = serde_json::from_str(message.into_text().unwrap().as_str())?;

        match request.trigger {
            PublishTrigger::StartTurn {
                active_client_id,
                user_name,
                dealer_card,
            } => {
                current_player_name = user_name.clone();
                if active_client_id.to_lowercase() == client_id.to_lowercase() {
                    println!();
                    println!("It's your turn!");
                    // TODO: The chips and bet amounts are not shared across clients.
                    start_turn(&mut socket, &mut me, dealer_card.unwrap());
                } else {
                    println!("It's {}'s turn.", current_player_name);
                    println!("Waiting for our turn...");
                }
            }
            PublishTrigger::CardsDrawn { cards } => {
                // TODO: The username does not populate for the none host player.
                print!("{} drew the following card(s): ", current_player_name);
                print_cards_in_hand(cards, None);
                println!();
            }
            PublishTrigger::RoundFinished(results) => {
                println!("The round has finished.");
                println!();
                for result in results {
                    if result.player.player_type == PlayerType::Dealer {
                        println!("The dealer's hand is: ");
                        print_cards_in_hand(result.player.hand, None);
                        println!("The dealer's hand value is: {}", result.player.hand_value);
                        println!();
                    } else if result.player.user_name.to_lowercase() == my_user_name.to_lowercase()
                    {
                        me = result.player.clone();
                        handle_bets(&me, &result.end_state, true);
                        me.current_bet = 0;
                    } else {
                        handle_bets(&result.player, &result.end_state, false);
                    }
                }

                // Start the next round.
                // If the game is over the server will let all clients know.
                if res.is_host {
                    let req = BlackjackRequest {
                        command: RequestCommand::Start,
                    };
                    send_request(req, &mut socket);
                }
            }
            PublishTrigger::GameFinished => {
                println!("The game has finished.");
                break;
            }
        }
    }

    Ok(())
}
