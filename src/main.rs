#![deny(non_snake_case)]
use ::rand::{rngs::SmallRng, Rng, SeedableRng};
use macroquad::prelude::*;
use macroquad::ui::{hash, root_ui};
use std::collections::{HashMap, VecDeque};

const GRID_W: usize = 64;
const GRID_H: usize = 1;
const HISTORY_LENGTH: usize = 16;
const NEIGHBORHOOD_W: usize = 3;
const NEIGHBORHOOD_H: usize = 1;
const MAX_NEIGHBORHOOD_BITS: usize = 16; // 2^16 = 65_536 combinations
const SPAWN_CHANCE: f32 = 0.2;

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
    spawn_chance: f32,
}

impl Automata {
    fn new(
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
            *c = if self.rng.gen_bool(self.spawn_chance as f64) {
                1
            } else {
                0
            };
        }
    }

    fn randomize_next(&mut self) {
        for c in self.cells_next.iter_mut() {
            *c = if self.rng.gen_bool(self.spawn_chance as f64) {
                1
            } else {
                0
            };
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
    neighborhood_w: usize,
    neighborhood_h: usize,
    input_grid_w: String,
    input_grid_h: String,
    input_neighborhood_w: String,
    input_neighborhood_h: String,
    input_history_length: String,
    show_history: bool,
    spawn_chance: f32,
    input_spawn_chance: String,
}

impl Game {
    fn new(
        grid_w: usize,
        grid_h: usize,
        history_length: usize,
        neighborhood_w: usize,
        neighborhood_h: usize,
        spawn_chance: f32,
    ) -> Self {
        let image = Image::gen_image_color(grid_w as u16, (grid_h * history_length) as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        let mut game = Self {
            automata: Automata::new(
                grid_w,
                grid_h,
                neighborhood_w,
                neighborhood_h,
                spawn_chance,
            ),
            image,
            texture,
            paused: false,
            time_since_last_step: 0.0f32,
            step_time: 0.05f32,
            cells_history: VecDeque::with_capacity(history_length),
            history_length,
            grid_w,
            grid_h,
            neighborhood_w,
            neighborhood_h,
            input_grid_w: grid_w.to_string(),
            input_grid_h: grid_h.to_string(),
            input_neighborhood_w: neighborhood_w.to_string(),
            input_neighborhood_h: neighborhood_h.to_string(),
            input_history_length: history_length.to_string(),
            show_history: true,
            spawn_chance,
            input_spawn_chance: format!("{:.2}", spawn_chance),
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
        self.cells_history = (0..self.history_length)
            .map(|_| vec![0u8; self.grid_w * self.grid_h])
            .collect();
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

    fn clamp_neighborhood_to_limit(&self, mut w: usize, mut h: usize) -> (usize, usize) {
        let max_bits: usize = MAX_NEIGHBORHOOD_BITS; // 2^bits combinations
        w = w.max(1);
        h = h.max(1);
        if w * h <= max_bits {
            return (w, h);
        }
        // shrink the larger dimension until the area fits.
        while w * h > max_bits {
            if w >= h && w > 1 {
                w -= 1;
            } else if h > 1 {
                h -= 1;
            } else {
                break;
            }
        }
        (w, h)
    }

    fn rebuild(
        &mut self,
        grid_w: usize,
        grid_h: usize,
        history_length: usize,
        neighborhood_w: usize,
        neighborhood_h: usize,
        spawn_chance: f32,
    ) {
        self.grid_w = grid_w.max(1);
        self.grid_h = grid_h.max(1);
        self.history_length = history_length.max(1);
        let (neighborhood_w, neighborhood_h) =
            self.clamp_neighborhood_to_limit(neighborhood_w, neighborhood_h);
        self.neighborhood_w = neighborhood_w;
        self.neighborhood_h = neighborhood_h;
        self.spawn_chance = spawn_chance.clamp(0.0, 1.0);

        self.automata = Automata::new(
            self.grid_w,
            self.grid_h,
            neighborhood_w,
            neighborhood_h,
            self.spawn_chance,
        );

        self.image = Image::gen_image_color(
            self.grid_w as u16,
            (self.grid_h * self.history_length) as u16,
            BLACK,
        );
        self.texture = Texture2D::from_image(&self.image);
        self.texture.set_filter(FilterMode::Nearest);

        self.reset();

        self.input_grid_w = self.grid_w.to_string();
        self.input_grid_h = self.grid_h.to_string();
        self.input_neighborhood_w = self.neighborhood_w.to_string();
        self.input_neighborhood_h = self.neighborhood_h.to_string();
        self.input_history_length = self.history_length.to_string();
        self.input_spawn_chance = format!("{:.2}", self.spawn_chance);
    }

    fn apply_inputs(&mut self) {
        let parse = |s: &str, fallback: usize| -> usize {
            s.trim()
                .parse::<usize>()
                .ok()
                .filter(|v| *v > 0)
                .unwrap_or(fallback)
        };
        let parse_f32 = |s: &str, fallback: f32| -> f32 {
            s.trim().parse::<f32>().ok().map(|v| v.clamp(0.0, 1.0)).unwrap_or(fallback)
        };

        let new_w = parse(&self.input_grid_w, self.grid_w);
        let new_h = parse(&self.input_grid_h, self.grid_h);
        let new_neighborhood_w = parse(&self.input_neighborhood_w, self.neighborhood_w);
        let new_neighborhood_h = parse(&self.input_neighborhood_h, self.neighborhood_h);
        let new_history_length = parse(&self.input_history_length, self.history_length);
        let new_spawn_chance = parse_f32(&self.input_spawn_chance, self.spawn_chance);

        self.rebuild(
            new_w,
            new_h,
            new_history_length,
            new_neighborhood_w,
            new_neighborhood_h,
            new_spawn_chance,
        );
    }

    fn draw_ui(&mut self) {
        let padding_y = 36.0;
        let width = 260.0;
        root_ui().window(
            hash!("controls"),
            vec2(12.0, padding_y),
            vec2(width, 280.0),
            |ui| {
                ui.label(None, "Board width");
                ui.input_text(hash!("grid_w"), "", &mut self.input_grid_w);
                ui.label(None, "Board height");
                ui.input_text(hash!("grid_h"), "", &mut self.input_grid_h);
                ui.label(None, "Neighborhood width");
                ui.input_text(hash!("nb_w"), "", &mut self.input_neighborhood_w);
                ui.label(None, "Neighborhood height");
                ui.input_text(hash!("nb_h"), "", &mut self.input_neighborhood_h);
                ui.label(None, "History length");
                ui.input_text(hash!("hist"), "", &mut self.input_history_length);
                ui.label(None, "Spawn chance (0-1)");
                ui.input_text(hash!("spawn"), "", &mut self.input_spawn_chance);

                if ui.button(None, "Apply (rebuild)") {
                    self.apply_inputs();
                }
            },
        );
        self.sanitize_inputs();
    }

    fn sanitize_inputs(&mut self) {
        let only_digits = |s: &mut String| s.retain(|c| c.is_ascii_digit());
        only_digits(&mut self.input_grid_w);
        only_digits(&mut self.input_grid_h);
        only_digits(&mut self.input_neighborhood_w);
        only_digits(&mut self.input_neighborhood_h);
        only_digits(&mut self.input_history_length);
        self.input_spawn_chance
            .retain(|c| c.is_ascii_digit() || c == '.');
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
        if is_key_pressed(KeyCode::Enter) {
            self.apply_inputs();
        }
        if is_key_pressed(KeyCode::H) {
            self.show_history = !self.show_history;
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
        self.draw_ui();

        clear_background(Color::from_rgba(12, 18, 28, 255));

        let win_w = screen_width();
        let win_h = screen_height();
        let rows_to_show = if self.show_history {
            (self.grid_h * self.history_length) as f32
        } else {
            self.grid_h as f32
        };
        let scale = (win_w / self.grid_w as f32).min(win_h / rows_to_show);
        let draw_w = self.grid_w as f32 * scale;
        let draw_h = rows_to_show * scale;
        let pos_x = ((win_w - draw_w) * 0.5).floor();
        let pos_y = ((win_h - draw_h) * 0.5).floor();

        let source_rect = if self.show_history {
            None
        } else {
            // Show the most recent snapshot (last history entry)
            let last_row = self
                .cells_history
                .len()
                .saturating_sub(1)
                .saturating_mul(self.grid_h);
            Some(Rect {
                x: 0.0,
                y: last_row as f32,
                w: self.grid_w as f32,
                h: self.grid_h as f32,
            })
        };

        draw_texture_ex(
            &self.texture,
            pos_x,
            pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                source: source_rect,
                ..Default::default()
            },
        );

        let info = format!(
            "Step: {:.3}s (Up/Down to adjust) | {} | View: {}",
            self.step_time,
            if self.paused { "Paused" } else { "Running" },
            if self.show_history { "History" } else { "Current" }
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
        SPAWN_CHANCE,
    );

    loop {
        game.step();
        game.draw();

        next_frame().await;
    }
}
