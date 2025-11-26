# Cellular Automata (Rust + macroquad)

Interactive 2D automata sandbox written in Rust using `macroquad`. You can tweak the board size, neighborhood size, and history depth at runtime, and regenerate states/rules on demand.

## Prerequisites
- Rust toolchain (`rustup` recommended)

## Run
```bash
cargo run
```

## Controls
- Space: pause/resume stepping
- Up/Down: speed up / slow down step time
- R: full reset (randomize rules and state, clear history)
- Enter: apply UI inputs (same as the Apply button)
- H: toggle between full history view and current board only

### UI Inputs (top-left)
- Board width / height
- Neighborhood width / height (clamped so `width * height <= 16`, i.e. <= 65,536 combinations)
- History length (number of past rows shown)
- Spawn chance (0-1) for initial/randomized cells
- Apply (rebuild): rebuilds automata, texture, and history with the entered values

## Code map
- `src/main.rs`: all game/automata logic, UI, rendering, and input handling.

## Notes
- Currently, we generate a unique output for every possible neighborhood combination. This can explode rather quickly so be careful getting this too high

## Future Features
- Allow user to create their own rules
