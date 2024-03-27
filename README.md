# blackjack-rs

## A cli based blackjack client and server

This mono-repo holds both the client and server code as well as a crate that is shared between the two.

### Server

The server crate listens out for the clients to register via http. Once registered the client is then assigned a web socket connection that is used for 2 way communication.

### Client

The client crate is a cli based app that will connect to the server and manage the user's input. The first client that connects will be considered the host and will be able to start the game.

The main loop of the client listens out for `PublishTriggers` sent from the server over the web socket and react accordingly.
