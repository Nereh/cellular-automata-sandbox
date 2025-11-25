# Cellular Automata Scaffold

Two starter tracks are available:
- Rust + macroquad (current)
- C++20 + CUDA + raylib (original scaffold) still lives in `CMakeLists.txt`, `src/main.cpp`, `src/kernels.cu`.

## Rust (macroquad)
Prerequisites:
- Rust toolchain (`rustup` recommended)

Run (debug):
```bash
cargo run
```

What it does:
- Opens a window via macroquad.
- Runs a CPU Game of Life stepper on a 256x144 grid.
- Updates a texture each frame and scales it to the window; simple controls are shown.

Controls:
- `Space`: pause/resume
- `R`: randomize the grid

Where to extend:
- `src/main.rs`: automata logic, rendering, input. Replace CPU stepping with GPU compute or change grid/visuals.

## C++ (CUDA + raylib)
Prerequisites:
- CUDA toolkit (nvcc + cudart)
- CMake 3.24+
- C++20 compiler
- raylib 5.0+ (or let CMake fetch it if network is allowed)

Build:
```bash
mkdir -p build
cd build
cmake -DCMAKE_BUILD_TYPE=Debug ..
cmake --build .
./ca_app
```

Notes:
- If CMake cannot find raylib, it will attempt to fetch from GitHub (requires network).
- CUDA sample kernel increments a small buffer and reports success in the window; check stderr if it fails.
