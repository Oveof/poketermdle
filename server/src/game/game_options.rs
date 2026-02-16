use std::collections::HashSet;

pub struct GameOptions {
    pub generations: Vec<i64>,
}
impl GameOptions {
    pub fn new(set: HashSet<i64>) -> Self {
        let generations = set.into_iter().collect::<Vec<i64>>();

        // .selected_gens
        // .clone()
        // .into_iter()
        // .collect::<Vec<i64>>()
        return Self { generations };
    }

    pub fn change_generations(&mut self, set: HashSet<i64>) {
        self.generations = set.into_iter().collect::<Vec<i64>>();
    }
}
