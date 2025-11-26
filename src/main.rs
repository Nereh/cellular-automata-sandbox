#![deny(non_snake_case)]
use ::rand::{rngs::SmallRng, Rng, SeedableRng};
use macroquad::prelude::*;
use std::collections::{HashMap, VecDeque};

const GRID_W: usize = 16;
const GRID_H: usize = 1;
const HISTORY_LENGTH: usize = 16;
const NEIGHBORHOOD_W: usize = 3;
const NEIGHBORHOOD_H: usize = 3;

#[derive(Clone)]
struct RulesCollection {
    hash_map: HashMap<u64, u8>,
}

impl RulesCollection {
    fn new() -> Self {
        Self {
            hash_map: HashMap::new(),
        }
    }

    fn randomize(&mut self, neighborhood_w: usize, neighborhood_h: usize, rng: &mut SmallRng) {
        let bits = neighborhood_w * neighborhood_h;
        assert!(bits <= 64, "neighborhood too big for u64 hash");

        let total_patterns: u128 = 1u128 << bits;

        self.hash_map.clear();

        for key in 0..total_patterns {
            let out = if rng.gen_bool(0.5) { 1u8 } else { 0u8 };

            self.hash_map.insert(key as u64, out);
        }
    }

    fn get_new_cell(&mut self, neighborhood: u64) -> u8 {
        return self.hash_map[&neighborhood];
    }
}

#[derive(Clone)]
struct Automata {
    rng: SmallRng,
    cells: Vec<u8>,
    cells_next: Vec<u8>,
    grid_h: usize,
    grid_w: usize,
    neighborhood_offsets: Vec<(isize, isize)>,
    rules_collection: RulesCollection,
    neighborhood_w: usize,
    neighborhood_h: usize,
}

impl Automata {
    fn new(grid_w: usize, grid_h: usize, neighborhood_w: usize, neighborhood_h: usize) -> Self {
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
        };
        automata.randomize();
        automata
    }

    fn step(&mut self) {
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

    fn cells(&self) -> Vec<u8> {
        self.cells.clone()
    }

    fn randomize_rules(&mut self) {
        self.rules_collection
            .randomize(self.neighborhood_w, self.neighborhood_h, &mut self.rng);
    }

    fn randomize(&mut self) {
        for c in self.cells.iter_mut() {
            *c = if self.rng.gen_bool(0.2) { 1 } else { 0 };
        }
    }

    fn randomize_next(&mut self) {
        for c in self.cells_next.iter_mut() {
            *c = if self.rng.gen_bool(0.2) { 1 } else { 0 };
        }
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

#[derive(Clone)]
struct Game {
    automata: Automata,
    image: Image,
    texture: Texture2D,
    paused: bool,
    time_since_last_step: f32,
    step_time: f32,
    cells_history: VecDeque<Vec<u8>>,
    history_length: usize,
    grid_w: usize,
    grid_h: usize,
}

impl Game {
    fn new(
        grid_w: usize,
        grid_h: usize,
        history_length: usize,
        neighborhood_w: usize,
        neighborhood_h: usize,
    ) -> Self {
        let image = Image::gen_image_color(grid_w as u16, (grid_h * history_length) as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        let mut game = Self {
            automata: Automata::new(grid_w, grid_h, neighborhood_w, neighborhood_h),
            image,
            texture,
            paused: false,
            time_since_last_step: 0.0f32,
            step_time: 0.05f32,
            cells_history: VecDeque::with_capacity(history_length),
            history_length,
            grid_w,
            grid_h,
        };
        game.init();
        game
    }

    fn add_history(&mut self) {
        if self.cells_history.len() == self.history_length {
            self.cells_history.pop_front();
        }
        self.cells_history.push_back(self.automata.cells());
    }

    fn init(&mut self) {
        self.cells_history = (0..self.history_length)
            .map(|_| vec![0u8; self.grid_w * self.grid_h])
            .collect();
        self.add_history();
    }

    fn reset(&mut self) {
        self.automata.randomize_rules();
        self.automata.randomize();
        self.cells_history.clear();
        self.add_history();
        self.time_since_last_step = 0.0;
    }

    fn randomize(&mut self) {
        self.automata.randomize();
    }

    fn update_texture(&mut self) {
        for (row_idx, history) in self.cells_history.iter().enumerate() {
            for y in 0..self.grid_h {
                for x in 0..self.grid_w {
                    let idx = y * self.grid_w + x;
                    let c = history[idx];
                    let color = if c == 1 { WHITE } else { BLACK };
                    self.image
                        .set_pixel(x as u32, (y + row_idx * self.grid_h) as u32, color);
                }
            }
        }

        self.texture.update(&self.image);
    }

    fn handle_input(&mut self) {
        let adjust = 0.005f32;
        if is_key_pressed(KeyCode::Space) {
            self.paused = !self.paused;
        }
        if is_key_down(KeyCode::Up) {
            self.step_time = (self.step_time - adjust).max(0.001);
        }
        if is_key_down(KeyCode::Down) {
            self.step_time = (self.step_time + adjust).min(5.0);
        }
        if is_key_pressed(KeyCode::R) {
            self.reset();
        }
    }

    fn step(&mut self) {
        self.handle_input();
        self.time_since_last_step += get_frame_time();
        if self.time_since_last_step < self.step_time || self.paused {
            return;
        }
        self.automata.step();
        self.time_since_last_step = 0.0;
        self.add_history();
    }

    fn draw(&mut self) {
        self.update_texture();

        clear_background(Color::from_rgba(12, 18, 28, 255));

        let win_w = screen_width();
        let win_h = screen_height();
        let total_rows = (self.grid_h * self.history_length) as f32;
        let scale = (win_w / self.grid_w as f32).min(win_h / total_rows);
        let draw_w = self.grid_w as f32 * scale;
        let draw_h = total_rows * scale;
        let pos_x = ((win_w - draw_w) * 0.5).floor();
        let pos_y = ((win_h - draw_h) * 0.5).floor();

        draw_texture_ex(
            &self.texture,
            pos_x,
            pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );

        let info = format!(
            "Step: {:.3}s (Up/Down to adjust) | {}",
            self.step_time,
            if self.paused { "Paused" } else { "Running" }
        );
        draw_text(&info, 12.0, 24.0, 20.0, LIGHTGRAY);
    }
}

#[macroquad::main("Hello World")]
async fn main() {
    let mut game = Game::new(
        GRID_W,
        GRID_H,
        HISTORY_LENGTH,
        NEIGHBORHOOD_W,
        NEIGHBORHOOD_H,
    );

    loop {
        game.step();
        game.draw();

        next_frame().await;
    }
}
