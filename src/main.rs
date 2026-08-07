mod caster;
mod framebuffer;
mod input;
mod line;
mod maze;
mod player;
mod render;

use caster::cast_fov_2d;
use framebuffer::Framebuffer;
use input::process_input;
use maze::{find_char, load_maze, validate_maze};
use minifb::{Key, Window, WindowOptions};
use player::Player;
use render::{maze_offset, render_maze, render_player};
use std::time::Instant;

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
    let mut player = Player::new(player_start.0, player_start.1, BLOCK_SIZE);

    println!("Maze loaded: {}x{}", maze_width, maze_height);
    println!("Player start: ({}, {})", player_start.0, player_start.1);
    println!("Goal: ({}, {})", goal.0, goal.1);
    println!(
        "Player world position: ({:.1}, {:.1})",
        player.pos.x, player.pos.y
    );
    println!("Player angle: {:.4} rad", player.a);
    println!("Player FOV: {:.4} rad", player.fov);

    let mut window = Window::new(
        "Proyecto 1 - Raycasting",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let (maze_offset_x, maze_offset_y) = maze_offset(&framebuffer, &maze, BLOCK_SIZE);
    let mut last_time = Instant::now();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let delta_time = current_time
            .duration_since(last_time)
            .as_secs_f32()
            .min(0.1);
        last_time = current_time;

        process_input(&window, &mut player, &maze, BLOCK_SIZE, delta_time);

        framebuffer.clear();
        render_maze(&mut framebuffer, &maze, BLOCK_SIZE);
        let rays = cast_fov_2d(
            &mut framebuffer,
            &maze,
            &player,
            BLOCK_SIZE,
            maze_offset_x,
            maze_offset_y,
        );
        let _ = rays.first().map(|ray| (ray.distance, ray.impact));
        render_player(&mut framebuffer, &player, maze_offset_x, maze_offset_y);

        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)?;
    }

    Ok(())
}
