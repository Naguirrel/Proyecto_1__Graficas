mod framebuffer;
mod line;
mod maze;
mod render;

use framebuffer::Framebuffer;
use maze::{find_char, load_maze, validate_maze};
use minifb::{Key, Window, WindowOptions};
use render::render_maze;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BLOCK_SIZE: usize = 40;

fn main() -> Result<(), minifb::Error> {
    let maze = load_maze("maze.txt");

    if !validate_maze(&maze) {
        panic!("Invalid maze: maze.txt must be rectangular and contain exactly one p and one g");
    }

    let maze_width = maze[0].len();
    let maze_height = maze.len();
    let player_start = find_char(&maze, 'p').expect("Maze does not contain player start");
    let goal = find_char(&maze, 'g').expect("Maze does not contain goal");

    println!("Maze loaded: {}x{}", maze_width, maze_height);
    println!("Player start: ({}, {})", player_start.0, player_start.1);
    println!("Goal: ({}, {})", goal.0, goal.1);

    let mut window = Window::new(
        "Proyecto 1 - Raycasting",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        framebuffer.clear();
        render_maze(&mut framebuffer, &maze, BLOCK_SIZE);

        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)?;
    }

    Ok(())
}
