use std::{collections::HashSet, env, io::Error as IoError, net::SocketAddr};

use futures_util::{SinkExt, StreamExt};
use poketermdle_common::*;
use rand::{
    Rng,
    distr::{Alphanumeric, SampleString},
};
use rustemon::{
    Follow,
    client::RustemonClient,
    evolution::{self, evolution_chain},
    model::{evolution::EvolutionChain, pokemon, resource::ApiResource},
    pokemon::pokemon_species,
};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::broadcast::error::RecvError,
};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::lobby::Lobby;

mod game;
mod lobby;
mod matchmaker;
mod player;
mod server;
mod websocket;

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:6767".to_string());

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    let mut lobby = Lobby::new().await;
    while let Ok((stream, addr)) = listener.accept().await {
        lobby.add_player(stream, addr).await;
        lobby.run().await;
        // lobby.players.push(::new(ws_stream, addr));
        // tokio::spawn(handle_connection_new(stream, addr));
    }
    // let client_builder = RustemonClient::new();
    //
    // let mut gens = HashSet::new();
    // gens.insert(1);
    // let state = GameState::new(gens).await;
    // println!("{:?}", state.current_pokemon.unwrap().name);

    Ok(())
}
