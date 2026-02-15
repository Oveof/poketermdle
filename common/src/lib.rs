use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct LobbyMessage {
    pub player_name: String,
    pub content: GameMessage,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameMessage {
    GuessResponse(GuessResponse),
    Guess(String),
    NewName(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GuessStatus {
    Correct(String),
    Incorrect(String),
    Partial(),
    Low(String),
    High(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GuessResponse {
    pub pokemon_name: GuessStatus,
    pub type_primary: GuessStatus,
    pub type_secondary: GuessStatus,
    pub habitats: Vec<GuessStatus>,
    pub generation: GuessStatus,
    pub evolution_stage: GuessStatus,
}
impl GuessResponse {
    pub fn new() -> Self {
        return Self {
            pokemon_name: GuessStatus::Incorrect("".into()),
            type_primary: GuessStatus::Incorrect("".into()),
            type_secondary: GuessStatus::Incorrect("".into()),
            habitats: Vec::new(),
            generation: GuessStatus::Incorrect("".into()),
            evolution_stage: GuessStatus::Incorrect("".into()),
        };
    }
}
