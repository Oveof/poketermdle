#[derive(Debug)]
pub struct Solution {
    pub current_pokemon: Option<rustemon::model::pokemon::Pokemon>,
    pub current_pokemon_species: Option<rustemon::model::pokemon::PokemonSpecies>,
    pub current_generation: Option<i64>,
    pub evolution_number: i64,
}
