use std::collections::HashSet;

use poketermdle_common::{GuessResponse, GuessStatus};
use rand::Rng;
use rustemon::{
    Follow,
    client::RustemonClient,
    model::{evolution::EvolutionChain, resource::ApiResource},
};

use crate::game::{game_options::GameOptions, solution::Solution};

pub struct GameState {
    pub client: RustemonClient,
    solution: Solution,
    options: GameOptions,
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
        let gen_index: usize = rng.random_range(0..self.options.generations.len());
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
