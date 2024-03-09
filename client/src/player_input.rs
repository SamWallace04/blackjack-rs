pub fn bet(chips: u32) -> u32 {
    println!("You have {} chips.", chips);
    println!("Place your bet: ");

    let bet = loop {
        let mut input = String::new();

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read the bet.");

        if let Ok(input_num) = input.trim().parse::<u32>() {
            if input_num < chips {
                break input_num;
            }
            println!("You don't have enough chips to cover that bet!");
            println!("Please enter a new bet: ");
        } else {
            println!("Please enter a vaild number: ")
        }
    };
    bet
}

pub fn get_player_action() -> PlayerAction {
    loop {
        let mut input = String::new();

        stdin()
            .read_line(&mut input)
            .expect("Could not read the input");

        let trimmed_input = input.trim().to_lowercase();

        let _ = match trimmed_input.as_str() {
            "hit" => return PlayerAction::Hit,
            "stand" => return PlayerAction::Stand,
            "double" => return PlayerAction::Double,
            _ => println!("Move not recognised. Please enter a vaild move:"),
        };
    }
}
