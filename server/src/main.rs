use std::{
    any::Any,
    collections::{HashMap, HashSet},
    env,
    io::Error as IoError,
    sync::Mutex,
};

use rand::Rng;
use rustemon::client::{RustemonClient, RustemonClientBuilder};
use tokio::net::TcpListener;

use crate::{
    game::GameState,
    websocket::{PeerMap, handle_connection},
};
mod game;
mod game_server;
mod server;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6767".to_string());

    let state = PeerMap::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        println!("New conn");
        tokio::spawn(handle_connection(state.clone(), stream, addr));
    }
    // let client_builder = RustemonClient::new();
    //
    // let mut gens = HashSet::new();
    // gens.insert(1);
    // let state = GameState::new(gens).await;
    // println!("{:?}", state.current_pokemon.unwrap().name);

    Ok(())
}
