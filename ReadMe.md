# Proyecto #1 Graficas

Raycasting project written in Rust for a computer graphics course. The program
loads a maze from `maze.txt`, renders a 2D map with rays, and can switch to a
simple first-person 3D projection.

## Requirements

- Rust toolchain with Cargo
- A desktop environment that can open a `minifb` window

## Run

```bash
cargo run
```

## Controls

- `W`: move forward
- `S`: move backward
- `A`: rotate left
- `D`: rotate right
- `Space`: jump
- `Tab`: switch between 2D and 3D render modes
- `Esc`: close the window

## Test

```bash
cargo test
```

## Project Structure

- `src/main.rs`: application loop, window setup, render mode switching
- `src/maze.rs`: maze loading, validation, tile lookup, and walkability rules
- `src/player.rs`: player position, angle, field of view, and jump state
- `src/input.rs`: keyboard handling and movement collision checks
- `src/caster.rs`: raycasting logic for 2D rays and 3D wall hits
- `src/render.rs`: 2D map rendering, 3D projection, and color mapping
- `src/framebuffer.rs`: pixel buffer abstraction used by the renderer
- `src/line.rs`: line drawing helper
- `maze.txt`: editable maze source

## Maze Format

The maze is a rectangular text file. It must contain exactly one player start
(`p`) and one goal (`g`). The outside border must be walls.

Known tile characters:

- `#`, `+`, `%`, `@`: walls with different colors
- space: walkable floor
- `p`: player start
- `g`: goal
