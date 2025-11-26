// THIS FILE WAS COMLPETELY CODEX SO I COULD SEE WHAT RUST LOOKS LIKE

#![deny(non_snake_case)]
use ::rand::{rngs::SmallRng, Rng, SeedableRng};
use macroquad::prelude::*;

const GRID_W: usize = 256;
const GRID_H: usize = 144;

fn window_conf() -> Conf {
    Conf {
        window_title: "Cellular Automata (macroquad scaffold)".into(),
        window_width: 960,
        window_height: 540,
        high_dpi: true,
        ..Default::default()
    }
}

#[derive(Clone)]
struct Automata {
    cells: Vec<u8>,
    scratch: Vec<u8>,
}

impl Automata {
    fn new() -> Self {
        let mut rng = SmallRng::from_entropy();
        let mut cells = vec![0u8; GRID_W * GRID_H];
        for c in &mut cells {
            *c = if rng.gen_bool(0.2) { 1 } else { 0 };
        }
        Self {
            scratch: cells.clone(),
            cells,
        }
    }

    fn step(&mut self) {
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let idx = y * GRID_W + x;
                let mut live = 0u8;
                for dy in [-1isize, 0, 1] {
                    for dx in [-1isize, 0, 1] {
                        if dx == 0 && dy == 0 {
                            continue;
                        }
                        let nx = (x as isize + dx + GRID_W as isize) as usize % GRID_W;
                        let ny = (y as isize + dy + GRID_H as isize) as usize % GRID_H;
                        live += self.cells[ny * GRID_W + nx];
                    }
                }
                let cell = self.cells[idx];
                let next = if cell == 1 && (live == 2 || live == 3) {
                    1
                } else if cell == 0 && live == 3 {
                    1
                } else {
                    0
                };
                self.scratch[idx] = next;
            }
        }
        self.cells.copy_from_slice(&self.scratch);
    }

    fn randomize(&mut self) {
        let mut rng = SmallRng::from_entropy();
        for c in &mut self.cells {
            *c = if rng.gen_bool(0.2) { 1 } else { 0 };
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut world = Automata::new();

    let mut image = Image::gen_image_color(GRID_W as u16, GRID_H as u16, BLACK);
    let mut texture = Texture2D::from_image(&image);
    texture.set_filter(FilterMode::Nearest);

    let mut paused = false;
    let mut last_step = 0.0f32;
    let step_time = 0.05f32; // seconds between steps

    loop {
        let dt = get_frame_time();
        last_step += dt;

        if is_key_pressed(KeyCode::Space) {
            paused = !paused;
        }
        if is_key_pressed(KeyCode::R) {
            world.randomize();
        }

        if !paused && last_step > step_time {
            world.step();
            last_step = 0.0;
        }

        // Write cells into the image buffer.
        for y in 0..GRID_H {
            for x in 0..GRID_W {
                let idx = y * GRID_W + x;
                let c = world.cells[idx];
                let color = if c == 1 { WHITE } else { BLACK };
                image.set_pixel(x as u32, y as u32, color);
            }
        }
        texture.update(&image);

        clear_background(Color::from_rgba(12, 18, 28, 255));

        let win_w = screen_width();
        let win_h = screen_height();
        let scale = (win_w / GRID_W as f32).min(win_h / GRID_H as f32);
        let draw_w = GRID_W as f32 * scale;
        let draw_h = GRID_H as f32 * scale;
        let pos_x = (win_w - draw_w) * 0.5;
        let pos_y = (win_h - draw_h) * 0.5;

        draw_texture_ex(
            &texture,
            pos_x,
            pos_y,
            WHITE,
            DrawTextureParams {
                dest_size: Some(vec2(draw_w, draw_h)),
                ..Default::default()
            },
        );

        draw_text(
            "Space: pause/resume | R: randomize",
            16.0,
            28.0,
            22.0,
            LIGHTGRAY,
        );
        draw_text(
            &format!(
                "FPS: {:.0} | State: {}",
                get_fps(),
                if paused { "Paused" } else { "Running" }
            ),
            16.0,
            52.0,
            22.0,
            LIGHTGRAY,
        );

        next_frame().await;
    }
}
