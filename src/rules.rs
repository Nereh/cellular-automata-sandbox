use ::rand::{rngs::SmallRng, Rng};
use std::collections::HashMap;

use crate::config::MAX_NEIGHBORHOOD_BITS;

#[derive(Clone)]
pub struct RulesCollection {
    hash_map: HashMap<u64, u8>,
}

impl RulesCollection {
    pub fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }

    pub fn randomize(&mut self, neighborhood_w: usize, neighborhood_h: usize, rng: &mut SmallRng) {
        let bits = neighborhood_w * neighborhood_h;
        assert!(
            bits <= MAX_NEIGHBORHOOD_BITS,
            "neighborhood too big for u64 hash"
        );

        let total_patterns: u128 = 1u128 << bits;

        self.hash_map.clear();

        for key in 0..total_patterns {
            let out = if rng.gen_bool(0.5) { 1u8 } else { 0u8 };
            self.hash_map.insert(key as u64, out);
        }
    }

    pub fn get_new_cell(&self, neighborhood: u64) -> u8 {
        self.hash_map[&neighborhood]
    }
}
