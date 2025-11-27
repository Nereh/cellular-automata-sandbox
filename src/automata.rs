use ::rand::{rngs::SmallRng, Rng, SeedableRng};

use crate::rules::RulesCollection;

#[derive(Clone)]
pub struct Automata {
    rng: SmallRng,
    cells: Vec<u8>,
    cells_next: Vec<u8>,
    grid_h: usize,
    grid_w: usize,
    neighborhood_offsets: Vec<(isize, isize)>,
    rules_collection: RulesCollection,
    neighborhood_w: usize,
    neighborhood_h: usize,
    spawn_chance: f32,
}

impl Automata {
    pub fn new(
        grid_w: usize,
        grid_h: usize,
        neighborhood_w: usize,
        neighborhood_h: usize,
        spawn_chance: f32,
    ) -> Self {
        let mut neighborhood_offsets = Vec::new();
        let start_x = -((neighborhood_w as isize - 1) / 2);
        let end_x = neighborhood_w as isize / 2;
        let start_y = -((neighborhood_h as isize - 1) / 2);
        let end_y = neighborhood_h as isize / 2;
        for dx in start_x..=end_x {
            for dy in start_y..=end_y {
                neighborhood_offsets.push((dx, dy));
            }
        }

        let mut rng = SmallRng::from_entropy();

        let mut rules_collection = RulesCollection::new();
        rules_collection.randomize(neighborhood_w, neighborhood_h, &mut rng);

        let mut automata = Self {
            rng,
            cells: vec![0u8; grid_w * grid_h],
            cells_next: vec![0u8; grid_w * grid_h],
            grid_h,
            grid_w,
            neighborhood_offsets,
            rules_collection,
            neighborhood_w,
            neighborhood_h,
            spawn_chance,
        };
        automata.randomize();
        automata
    }

    pub fn step(&mut self) {
        for x in 0..self.grid_w {
            for y in 0..self.grid_h {
                let idx = x + y * self.grid_w;
                let neighborhood = self.get_neighborhood_hash(x, y);
                self.cells_next[idx] = self.rules_collection.get_new_cell(neighborhood);
            }
        }
        std::mem::swap(&mut self.cells, &mut self.cells_next);
        self.cells_next.fill(0);
    }

    pub fn cells(&self) -> Vec<u8> {
        self.cells.clone()
    }

    pub fn randomize_rules(&mut self) {
        self.rules_collection
            .randomize(self.neighborhood_w, self.neighborhood_h, &mut self.rng);
    }

    pub fn randomize(&mut self) {
        for c in self.cells.iter_mut() {
            *c = if self.rng.gen_bool(self.spawn_chance as f64) {
                1
            } else {
                0
            };
        }
    }

    pub fn randomize_next(&mut self) {
        for c in self.cells_next.iter_mut() {
            *c = if self.rng.gen_bool(self.spawn_chance as f64) {
                1
            } else {
                0
            };
        }
    }

    pub fn set_spawn_chance(&mut self, spawn_chance: f32) {
        self.spawn_chance = spawn_chance.clamp(0.0, 1.0);
    }

    fn get_neighborhood_hash(&mut self, x: usize, y: usize) -> u64 {
        self.neighborhood_offsets
            .iter()
            .enumerate()
            .map(|(cell_index, (dx, dy))| {
                let neighbor_x = (x as isize + dx).rem_euclid(self.grid_w as isize);
                let neighbor_y = (y as isize + dy).rem_euclid(self.grid_h as isize);
                let idx = (neighbor_x + neighbor_y * self.grid_w as isize) as usize;
                (self.cells[idx] as u64) << cell_index
            })
            .fold(0u64, |acc, val| acc | val)
    }
}
