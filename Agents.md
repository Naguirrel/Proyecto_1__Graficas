# Agents.md

Guidance for coding agents working in this repository.

## Project Context

This is a Rust raycasting project using `minifb` for window output. The program
loads `maze.txt`, validates it, and renders either a 2D ray view or a simple 3D
projection.

## Commands

Run the app:

```bash
cargo run
```

Run tests:

```bash
cargo test
```

Format code:

```bash
cargo fmt
```

## Working Guidelines

- Keep changes scoped to the requested behavior.
- Prefer existing modules over introducing new abstractions.
- Preserve the text-based maze workflow in `maze.txt`.
- Keep rendering code deterministic and easy to test where possible.
- Use `cargo fmt` after Rust source edits.
- Run `cargo test` when changing movement, maze validation, raycasting, or
  projection behavior.

## Important Files

- `src/main.rs`: main loop and render mode state
- `src/input.rs`: keyboard controls and movement
- `src/maze.rs`: maze parsing, validation, and collision helpers
- `src/caster.rs`: ray intersection logic
- `src/render.rs`: color mapping and 2D/3D rendering
- `src/player.rs`: player state and jump physics
- `maze.txt`: map layout used at runtime and in tests

## Maze Rules

The maze must be rectangular, fully bordered by wall characters, and contain
exactly one `p` and one `g`. Walkable cells are space, `p`, and `g`.

Wall characters are `#`, `+`, `%`, `@`, `&`, and `!`. The normal wall colors
are `#`, `+`, `%`, `@`, and `&`; adjacent wall cells must not use the same
character/color. The yellow wall character `!` is reserved only for walls
directly next to the goal.
