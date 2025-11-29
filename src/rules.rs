use ::rand::{rngs::SmallRng, Rng, SeedableRng};
use std::collections::HashMap;

use crate::config::MAX_NEIGHBORHOOD_BITS;

pub trait Rule {
    fn apply(&self, neighborhood: u64) -> Option<u8>;
}

#[derive(Clone)]
pub struct HashRule {
    hash_map: HashMap<u64, u8>,
    neighborhood_w: usize,
    neighborhood_h: usize,
}

impl Rule for HashRule {
    fn apply(&self, neighborhood: u64) -> Option<u8> {
        self.hash_map.get(&neighborhood).copied()
    }
}

impl HashRule {
    pub fn new(neighborhood_w: usize, neighborhood_h: usize) -> Self {
        Self {
            hash_map: HashMap::new(),
            neighborhood_w,
            neighborhood_h,
        }
    }

    pub fn set_neighborhood_size(&mut self, neighborhood_w: usize, neighborhood_h: usize) {
        self.neighborhood_w = neighborhood_w;
        self.neighborhood_h = neighborhood_h;
    }

    pub fn add_mapping(&mut self, neighborhood: u64, value: u8) {
        self.hash_map.insert(neighborhood, value);
    }

    pub fn reset(&mut self) {
        self.hash_map.clear();
    }

    pub fn fill_random(&mut self, rng: &mut SmallRng, num_keys: usize) {
        let bits = self.neighborhood_w * self.neighborhood_h;

        let mask: u64 = if bits >= 64 {
            u64::MAX
        } else {
            (1u64 << bits) - 1
        };

        self.hash_map.clear();
        while self.hash_map.len() < num_keys {
            let key = rng.gen::<u64>() & mask;
            let val = if rng.gen_bool(0.5) { 1u8 } else { 0u8 };
            self.hash_map.entry(key).or_insert(val);
        }
    }
}

#[derive(Clone)]
pub struct RulesCollection {
    neighborhood_w: usize,
    neighborhood_h: usize,
    rules: Vec<HashRule>,
}

impl RulesCollection {
    pub fn new(neighborhood_w: usize, neighborhood_h: usize) -> Self {
        let mut this = Self {
            neighborhood_w,
            neighborhood_h,
            rules: Vec::new(),
        };
        let mut rng = SmallRng::from_entropy();
        this.add_rule(HashRule::new(neighborhood_w, neighborhood_h));
        this.randomize(&mut rng);
        this
    }

    pub fn add_rule(&mut self, rule: HashRule) {
        self.rules.push(rule);
    }

    pub fn set_neighborhood_size(
        &mut self,
        neighborhood_w: usize,
        neighborhood_h: usize,
        rng: &mut SmallRng,
    ) {
        self.neighborhood_w = neighborhood_w;
        self.neighborhood_h = neighborhood_h;
        for rule in &mut self.rules {
            rule.set_neighborhood_size(neighborhood_w, neighborhood_h);
        }
        self.randomize(rng);
    }

    pub fn randomize(&mut self, rng: &mut SmallRng) {
        let bits = self.neighborhood_w * self.neighborhood_h;

        let total_patterns = 1usize << bits;

        for rule in &mut self.rules {
            rule.fill_random(rng, total_patterns);
        }
    }

    pub fn get_new_cell(&self, neighborhood: u64) -> u8 {
        for rule in &self.rules {
            if let Some(v) = rule.apply(neighborhood) {
                return v;
            }
        }
        0
    }
}
