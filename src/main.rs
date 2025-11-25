#![deny(non_snake_case)]
use ::rand::{rngs::SmallRng, Rng, SeedableRng};
use macroquad::prelude::*;
use std::collections::VecDeque;

const GRID_W: usize = 128;
const GRID_H: usize = 16;

#[derive(Clone)]
struct Game {
    rng: SmallRng,
    cells: Vec<u8>,
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
    fn new() -> Self {
        let history_length = 16;
        let image = Image::gen_image_color(GRID_W as u16, (GRID_H * history_length) as u16, BLACK);
        let texture = Texture2D::from_image(&image);
        texture.set_filter(FilterMode::Nearest);

        let mut game = Self {
            rng: SmallRng::from_entropy(),
            cells: vec![0u8; GRID_W * GRID_H],
            image,
            texture,
            paused: false,
            time_since_last_step: 0.0f32,
            step_time: 0.05f32,
            cells_history: VecDeque::with_capacity(history_length),
            history_length,
            grid_w: GRID_W,
            grid_h: GRID_H,
        };
        game.init();
        game
    }

    fn add_history(&mut self) {
        self.cells_history.push_back(self.cells.clone());
        self.cells_history.pop_front();
    }

    fn init(&mut self) {
        self.cells_history = (0..self.history_length)
            .map(|_| vec![0u8; self.grid_w * self.grid_h])
            .collect();
        self.randomize();
        self.add_history();
    }

    fn randomize(&mut self) {
        for c in self.cells.iter_mut() {
            *c = if self.rng.gen_bool(0.2) { 1 } else { 0 };
        }
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

    fn step(&mut self) {
        self.time_since_last_step += get_frame_time();
        if self.time_since_last_step < self.step_time || self.paused {
            return;
        }
        self.randomize();
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
    }
}

#[macroquad::main("Hello World")]
async fn main() {
    let mut game = Game::new();

    loop {
        game.step();
        game.draw();

        next_frame().await;
    }
}
