#![deny(non_snake_case)]

mod automata;
mod config;
mod game;
mod rules;

use config::{GRID_H, GRID_W, HISTORY_LENGTH, NEIGHBORHOOD_H, NEIGHBORHOOD_W, SPAWN_CHANCE};
use game::Game;
use macroquad::prelude::*;

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
