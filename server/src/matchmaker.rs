use std::collections::HashMap;

use tokio::net::TcpListener;

use crate::lobby::Lobby;

type MMReceiver = tokio::sync::mpsc::Receiver<>

pub struct Matchmaker {
    lobbies: HashMap<String, Lobby>,
}
impl Matchmaker {
    pub async fn run(listener: TcpListener) {
        while let Ok((stream, addr)) = listener.accept().await {
            lobby.add_player(stream, addr).await;
            lobby.run().await;
        }
    }
}
