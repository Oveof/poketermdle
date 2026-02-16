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

mod game;
mod game_server;
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

struct Lobby {
    name: String,
    players: Vec<String>,
    game_state: GameState,
    rx: tokio::sync::mpsc::Receiver<LobbyMessage>,
    tx: tokio::sync::broadcast::Sender<LobbyMessage>,
    player_tx: tokio::sync::mpsc::Sender<LobbyMessage>,
}
impl Lobby {
    async fn new() -> Self {
        let game_state = GameState::new(HashSet::from([1])).await;
        let (tx, rx) = tokio::sync::mpsc::channel::<LobbyMessage>(100);
        let (broadcast_sender, _) = tokio::sync::broadcast::channel::<LobbyMessage>(100);

        return Lobby {
            name: "Default".into(),
            players: Vec::new(),
            game_state,
            rx,
            tx: broadcast_sender,
            player_tx: tx,
        };
    }
    async fn add_player(&mut self, stream: TcpStream, addr: SocketAddr) {
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut player = Player::new(ws_stream, addr, self.player_tx.clone(), self.tx.subscribe());
        self.players.push(player.name.clone());
        tokio::spawn(async move {
            player.start_listener().await;
        });
    }
    async fn run(&mut self) {
        //Init, who starts etc?
        let mut rng = rand::thread_rng();
        let mut player_index: usize = rng.random_range(0..self.players.len());
        let mut current_player = self.players.get(player_index).unwrap();
        while let Some(msg) = self.rx.recv().await {
            if msg.player_name != *current_player {
                println!("Incorrect player turn");
                continue;
            }
            match msg.content {
                GameMessage::Guess(pokemon_name) => {
                    let guess_response = self
                        .game_state
                        .guess(pokemon_name.clone(), &current_player)
                        .await;
                    println!(
                        "player: {} guessed: {} with response: {:?}",
                        &msg.player_name, pokemon_name, &guess_response
                    );
                    let _ = self.tx.send(LobbyMessage {
                        player_name: current_player.clone(),
                        content: GameMessage::GuessResponse(guess_response),
                    });
                }
                _ => {
                    println!("Not implemented lol");
                }
            }
            player_index = (player_index + 1) % self.players.len();
            //FIXME if someone leaves this shit can get fucked
            current_player = self.players.get(player_index).unwrap();
        }
    }
}

struct Player {
    name: String,
    stream: WebSocketStream<TcpStream>,
    addr: SocketAddr,
    game_tx: tokio::sync::mpsc::Sender<LobbyMessage>,
    game_rx: tokio::sync::broadcast::Receiver<LobbyMessage>,
}

enum Event {
    Ws(Option<Result<Message, tokio_tungstenite::tungstenite::Error>>),
    Lobby(Result<LobbyMessage, RecvError>),
}
impl Player {
    fn new(
        stream: WebSocketStream<TcpStream>,
        addr: SocketAddr,
        game_tx: tokio::sync::mpsc::Sender<LobbyMessage>,
        game_rx: tokio::sync::broadcast::Receiver<LobbyMessage>,
    ) -> Self {
        let name = Alphanumeric.sample_string(&mut rand::rng(), 16);
        return Self {
            name,
            stream,
            addr,
            game_tx,
            game_rx,
        };
    }
    pub async fn start_listener(&mut self) {
        loop {
            let event = tokio::select! {
                res = self.stream.next() => Event::Ws(res),
                lobby_message = self.game_rx.recv() => Event::Lobby(lobby_message),
            };

            match event {
                Event::Ws(msg) => match msg.unwrap().unwrap() {
                    Message::Text(txt) => {
                        self.game_tx
                            .send(LobbyMessage {
                                player_name: self.name.clone(),
                                content: GameMessage::Guess(txt.to_string().trim_end().to_string()),
                            })
                            .await;
                    }
                    _ => {
                        println!("Other data types are not real they cant hurt me");
                    }
                },
                Event::Lobby(msg) => {
                    let encoded_msg = bincode::serialize(&msg.unwrap()).unwrap();
                    let _ = self.stream.send(Message::Binary(encoded_msg.into())).await;
                    let _ = self.stream.flush().await;
                }
            }
        }
    }
}

pub async fn handle_connection_new(raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    let (mut outgoing, mut incoming) = ws_stream.split();

    while let Some(msg) = incoming.next().await {
        println!("{}", msg.unwrap());
        let out_msg = outgoing.send(Message::Text("pong".into())).await;
        if out_msg.is_err() {
            println!("shit fucked");
        }
    }

    println!("{} disconnected", &addr);
}

#[derive(Debug)]
struct Solution {
    pub current_pokemon: Option<rustemon::model::pokemon::Pokemon>,
    pub current_pokemon_species: Option<rustemon::model::pokemon::PokemonSpecies>,
    pub current_generation: Option<i64>,
    pub evolution_number: i64,
}
struct GameState {
    pub client: RustemonClient,
    solution: Solution,
    options: GameOptions,
}
struct GameOptions {
    generations: Vec<i64>,
}
impl GameState {
    pub async fn new(generations: HashSet<i64>) -> Self {
        let mut state = GameState {
            client: rustemon::client::RustemonClient::default(),
            solution: Solution {
                current_pokemon: None,
                current_pokemon_species: None,
                current_generation: None,
                evolution_number: 0,
            },
            options: GameOptions::new(generations),
        };
        state.new_pokemon().await;

        return state;
    }
    async fn get_evolution_number(
        &mut self,
        pokemon_name: &str,
        resource: &ApiResource<EvolutionChain>,
    ) -> i64 {
        let evolution_chain = resource.follow(&self.client).await;
        if evolution_chain.is_err() {
            println!("{:?}", evolution_chain);
        }
        if evolution_chain.as_ref().unwrap().chain.species.name == pokemon_name {
            return 1;
        }
        for (num, evolution_next) in evolution_chain.unwrap().chain.evolves_to.iter().enumerate() {
            if evolution_next.species.name == pokemon_name {
                return 2 as i64;
            }
            for final_evolution in evolution_next.evolves_to.iter() {
                if final_evolution.species.name == pokemon_name {
                    return 3 as i64;
                }
            }
        }
        return -1;
    }
    pub async fn guess(&mut self, pokemon_name: String, player_name: &str) -> GuessResponse {
        let pokemon =
            match rustemon::pokemon::pokemon::get_by_name(&pokemon_name, &self.client).await {
                Ok(p) => p,
                Err(e) => {
                    return GuessResponse::new();
                }
            };
        let pokemon_species =
            rustemon::pokemon::pokemon_species::get_by_name(&pokemon_name, &self.client)
                .await
                .unwrap();

        let guessed_generation = rustemon::games::generation::get_by_name(
            &pokemon_species.generation.name,
            &self.client,
        )
        .await
        .unwrap();
        let evolution_number = self
            .get_evolution_number(
                &pokemon_name,
                pokemon_species.evolution_chain.as_ref().unwrap(),
            )
            .await;

        let current_generation = self.solution.current_generation.unwrap();

        // let pokemon = rustemon::pokemon::pokemon::get_by_name(&pokemon_name, &self.client)
        //     .await
        //     .unwrap();

        let mut name_response = GuessStatus::Incorrect("".into());
        let mut type_primary_response = GuessStatus::Incorrect("".into());
        let mut type_secondary_response = GuessStatus::Incorrect("".into());
        let mut habitats_response = vec![GuessStatus::Incorrect("".into())];
        let mut generation_response = GuessStatus::Incorrect("".into());
        let mut evolution_response = GuessStatus::Incorrect("".into());

        if pokemon.name == self.solution.current_pokemon.as_ref().unwrap().name {
            name_response = GuessStatus::Correct(pokemon_name);
        }

        if guessed_generation.id < current_generation {
            generation_response = GuessStatus::Low("".into());
        } else if guessed_generation.id > current_generation {
            generation_response = GuessStatus::High("".into());
        } else {
            generation_response = GuessStatus::Correct("".into());
        }

        println!("evo guessed: {}", evolution_number);
        if evolution_number < self.solution.evolution_number {
            evolution_response = GuessStatus::Low("".into());
        } else if evolution_number > self.solution.evolution_number {
            evolution_response = GuessStatus::High("".into());
        } else {
            evolution_response = GuessStatus::Correct("".into());
        }

        if &self
            .solution
            .current_pokemon
            .as_ref()
            .unwrap()
            .types
            .get(0)
            .unwrap()
            == &pokemon.types.get(0).unwrap()
        {
            type_primary_response = GuessStatus::Correct("".into());
        }

        if self.solution.current_pokemon.as_ref().unwrap().types.len() == 2
            && pokemon.types.len() == 2
        {
            if self
                .solution
                .current_pokemon
                .as_ref()
                .unwrap()
                .types
                .get(1)
                .unwrap()
                == pokemon.types.get(1).unwrap()
            {
                type_secondary_response = GuessStatus::Correct("".into());
            }
        }

        // if pokemon.try_into == self.current_pokemon.as_ref().unwrap().name {
        //     name_response = GuessStatus::Correct(pokemon_name);
        // }

        return GuessResponse {
            pokemon_name: name_response,
            type_primary: type_primary_response,
            type_secondary: type_secondary_response,
            habitats: habitats_response,
            generation: generation_response,
            evolution_stage: evolution_response,
        };
    }
    async fn new_pokemon(&mut self) {
        let mut rng = rand::thread_rng();
        let gen_index: usize = rng.gen_range(0..self.options.generations.len());
        let gen_num = *self.options.generations.get(gen_index).unwrap();

        let generation = rustemon::games::generation::get_by_id(gen_num, &self.client)
            .await
            .unwrap();
        println!("Generation: {}", generation.name);

        let pokemon_num = rng.random_range(0..=generation.pokemon_species.len());

        let pokemon = generation.pokemon_species.get(pokemon_num).unwrap();
        let pokemon_species = pokemon.follow(&self.client).await.unwrap();

        let pokemon_name = pokemon.name.clone();

        self.solution.current_generation = Some(generation.id);
        self.solution.current_pokemon_species = Some(pokemon_species.clone());
        self.solution.current_pokemon = Some(
            rustemon::pokemon::pokemon::get_by_name(&pokemon_name, &self.client)
                .await
                .unwrap(),
        );
        let chain_resource = pokemon_species.evolution_chain.as_ref().unwrap();

        self.solution.evolution_number = self
            .get_evolution_number(&pokemon_name, chain_resource)
            .await;

        println!(
            "solution: {} evo:{:?}",
            &self.solution.current_pokemon.as_ref().unwrap().name,
            &self.solution.evolution_number
        );
    }
}

impl GameOptions {
    fn new(set: HashSet<i64>) -> Self {
        let generations = set.into_iter().collect::<Vec<i64>>();

        // .selected_gens
        // .clone()
        // .into_iter()
        // .collect::<Vec<i64>>()
        return Self { generations };
    }

    fn change_generations(&mut self, set: HashSet<i64>) {
        self.generations = set.into_iter().collect::<Vec<i64>>();
    }
}
