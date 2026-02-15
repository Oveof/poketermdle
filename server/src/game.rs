// use std::{collections::HashSet, hash::Hash};
//
// use rand::Rng;
// use rustemon::{self, client::RustemonClient};
//
// pub struct GameState {
//     client: RustemonClient,
//     pub current_pokemon: Option<rustemon::model::pokemon::Pokemon>,
//     options: GameOptions,
// }
//
// pub enum GameMessage {}
// pub enum GuessStatus {
//     Correct(String),
//     Incorrect(String),
//     Low(String),
//     High(String),
// }
//
// pub struct GuessResponse {
//     pokemon_name: GuessStatus,
//     type_primary: GuessStatus,
//     type_secondary: GuessStatus,
//     habitats: Vec<GuessStatus>,
//     generation: GuessStatus,
// }
//
// impl GameState {
//     pub async fn new(generations: HashSet<i64>) -> Self {
//         let mut state = GameState {
//             client: rustemon::client::RustemonClient::default(),
//             current_pokemon: None,
//             options: GameOptions::new(generations),
//         };
//         state.new_pokemon().await;
//
//         return state;
//     }
//     pub async fn guess(&self, pokemon_name: String) -> GuessResponse {
//         let pokemon = rustemon::pokemon::pokemon::get_by_name(&pokemon_name, &self.client)
//             .await
//             .unwrap();
//
//         let mut name_response = GuessStatus::Incorrect("".into());
//         let mut type_primary_response = GuessStatus::Incorrect("".into());
//         let mut type_secondary_response = GuessStatus::Incorrect("".into());
//         let mut habitats_response = vec![GuessStatus::Incorrect("".into())];
//         let mut generation_response = GuessStatus::Incorrect("".into());
//
//         if pokemon.name == self.current_pokemon.as_ref().unwrap().name {
//             name_response = GuessStatus::Correct(pokemon_name);
//         }
//         // if pokemon.gen == self.current_pokemon.as_ref().unwrap().name {
//         //     name_response = GuessStatus::Correct(pokemon_name);
//         // }
//
//         return GuessResponse {
//             pokemon_name: name_response,
//             type_primary: type_primary_response,
//             type_secondary: type_secondary_response,
//             habitats: habitats_response,
//             generation: generation_response,
//         };
//     }
//     async fn new_pokemon(&mut self) {
//         let mut rng = rand::thread_rng();
//         let gen_index: usize = rng.gen_range(0..self.options.generations.len());
//         let gen_num = *self.options.generations.get(gen_index).unwrap();
//
//         let generation = rustemon::games::generation::get_by_id(gen_num, &self.client)
//             .await
//             .unwrap();
//         println!("Generation: {}", generation.name);
//
//         let pokemon_num = rng.gen_range(0..=generation.pokemon_species.len());
//
//         let pokemon_name = generation
//             .pokemon_species
//             .get(pokemon_num)
//             .unwrap()
//             .name
//             .clone();
//
//         self.current_pokemon = Some(
//             rustemon::pokemon::pokemon::get_by_name(&pokemon_name, &self.client)
//                 .await
//                 .unwrap(),
//         );
//     }
// }
// struct GameOptions {
//     generations: Vec<i64>,
// }
// impl GameOptions {
//     fn new(set: HashSet<i64>) -> Self {
//         let generations = set.into_iter().collect::<Vec<i64>>();
//
//         // .selected_gens
//         // .clone()
//         // .into_iter()
//         // .collect::<Vec<i64>>()
//         return Self { generations };
//     }
//
//     fn change_generations(&mut self, set: HashSet<i64>) {
//         self.generations = set.into_iter().collect::<Vec<i64>>();
//     }
// }
