mod caster;
mod framebuffer;
mod game;
mod input;
mod line;
mod maze;
mod player;
mod render;

use caster::cast_fov_2d;
use framebuffer::Framebuffer;
use game::{GameState, player_reached_goal, reset_player};
use input::process_input;
use maze::{find_char, load_maze, validate_maze};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use player::Player;
use render::{maze_offset, render_3d, render_maze, render_player, render_victory_screen};
use std::time::Instant;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BLOCK_SIZE: usize = 40;

#[derive(Clone, Copy)]
enum RenderMode {
    Mode2D,
    Mode3D,
}

impl RenderMode {
    fn toggle(self) -> Self {
        match self {
            Self::Mode2D => Self::Mode3D,
            Self::Mode3D => Self::Mode2D,
        }
    }
}

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
    let mut render_mode = RenderMode::Mode2D;
    let mut game_state = GameState::Playing;

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let delta_time = current_time
            .duration_since(last_time)
            .as_secs_f32()
            .min(0.1);
        last_time = current_time;

        match game_state {
            GameState::Playing => {
                process_input(&window, &mut player, &maze, BLOCK_SIZE, delta_time);

                if player_reached_goal(&maze, &player, BLOCK_SIZE) {
                    game_state = GameState::Won;
                } else if window.is_key_pressed(Key::Tab, KeyRepeat::No) {
                    render_mode = render_mode.toggle();
                }
            }
            GameState::Won => {
                if window.is_key_pressed(Key::R, KeyRepeat::No) {
                    reset_player(&mut player, player_start, BLOCK_SIZE);
                    game_state = GameState::Playing;
                }
            }
        }

        framebuffer.clear();

        match game_state {
            GameState::Playing => match render_mode {
                RenderMode::Mode2D => {
                    render_maze(&mut framebuffer, &maze, BLOCK_SIZE);
                    cast_fov_2d(
                        &mut framebuffer,
                        &maze,
                        &player,
                        BLOCK_SIZE,
                        maze_offset_x,
                        maze_offset_y,
                    );
                    render_player(&mut framebuffer, &player, maze_offset_x, maze_offset_y);
                }
                RenderMode::Mode3D => {
                    render_3d(&mut framebuffer, &maze, &player, BLOCK_SIZE);
                }
            },
            GameState::Won => {
                render_victory_screen(&mut framebuffer);
            }
        }

        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)?;
    }

    Ok(())
}
