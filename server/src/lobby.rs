use std::{collections::HashSet, net::SocketAddr};

use poketermdle_common::{GameMessage, LobbyMessage};
use rand::Rng;
use tokio::net::TcpStream;

use crate::{game::game_state::GameState, player::Player};

pub struct Lobby {
    name: String,
    players: Vec<String>,
    game_state: GameState,
    rx: tokio::sync::mpsc::Receiver<LobbyMessage>,
    tx: tokio::sync::broadcast::Sender<LobbyMessage>,
    player_tx: tokio::sync::mpsc::Sender<LobbyMessage>,
}
impl Lobby {
    pub async fn new() -> Self {
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
    pub async fn add_player(&mut self, stream: TcpStream, addr: SocketAddr) {
        let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
        let mut player = Player::new(ws_stream, addr, self.player_tx.clone(), self.tx.subscribe());
        self.players.push(player.name.clone());
        tokio::spawn(async move {
            player.start_listener().await;
        });
    }
    pub async fn run(&mut self) {
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
                GameMessage::JoinLobby(a) => {}
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
