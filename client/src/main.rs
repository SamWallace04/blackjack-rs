use color_eyre::eyre::Result;
use tungstenite::{connect, Message};
use url::Url;

fn main() -> Result<()> {
    color_eyre::install()?;

    let (mut socket, response) =
        connect(Url::parse("ws://localhost:7878/socket").unwrap()).expect("Can't connect");

    println!("Connected to the server");
    println!("Response HTTP code: {}", response.status());

    socket
        .send(Message::Text("Hello WebSocket".into()))
        .unwrap();
    let msg = socket.read().expect("Error reading message");
    println!("Received: {}", msg);

    socket.send(Message::Text("bye".into())).unwrap();

    let new_msg = socket.read().unwrap();
    println!("new msg {}", new_msg);

    socket.close(None).unwrap();
    socket.flush().unwrap();

    Ok(())
}
