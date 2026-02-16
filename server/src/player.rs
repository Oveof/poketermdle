use std::net::SocketAddr;

use futures_util::{SinkExt, StreamExt};
use poketermdle_common::{GameMessage, LobbyMessage};
use rand::distr::{Alphanumeric, SampleString};
use tokio::{net::TcpStream, sync::broadcast::error::RecvError};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

pub struct Player {
    pub name: String,
    pub stream: WebSocketStream<TcpStream>,
    pub addr: SocketAddr,
    pub game_tx: tokio::sync::mpsc::Sender<LobbyMessage>,
    pub game_rx: tokio::sync::broadcast::Receiver<LobbyMessage>,
}

enum Event {
    Ws(Option<Result<Message, tokio_tungstenite::tungstenite::Error>>),
    Lobby(Result<LobbyMessage, RecvError>),
}
impl Player {
    pub fn new(
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
