mod caster;
mod framebuffer;
mod game;
mod gamepad;
mod input;
mod line;
mod maze;
mod player;
mod render;
pub mod texture;

use caster::cast_fov_2d;
use framebuffer::Framebuffer;
use game::{GameState, player_reached_goal, reset_player};
use gamepad::GamepadInput;
use input::{MouseLook, process_input};
use maze::{Maze, find_char, load_maze, validate_maze};
use minifb::{Key, KeyRepeat, Window, WindowOptions};
use player::Player;
use render::{
    maze_offset, render_3d, render_fps_overlay, render_maze, render_minimap, render_player,
    render_victory_screen, render_welcome_screen,
};
use std::time::Instant;
use texture::TextureManager;

const WIDTH: usize = 800;
const HEIGHT: usize = 600;
const BLOCK_SIZE: usize = 40;
const FPS_UPDATE_INTERVAL: f32 = 0.5;
const LEVEL_PATHS: [&str; 3] = ["maze.txt", "maze_2.txt", "maze_3.txt"];

struct Level {
    maze: Maze,
    player_start: (usize, usize),
}

impl Level {
    fn load(path: &str) -> Self {
        let maze = load_maze(path);

        if !validate_maze(&maze) {
            panic!(
                "Invalid maze: {path} must be rectangular, bordered by walls, and contain exactly one p and one g"
            );
        }

        let maze_width = maze[0].len();
        let maze_height = maze.len();
        let player_start = find_char(&maze, 'p').expect("Maze does not contain player start");
        let goal = find_char(&maze, 'g').expect("Maze does not contain goal");

        println!("Level loaded from {path}: {maze_width}x{maze_height}");
        println!("Player start: ({}, {})", player_start.0, player_start.1);
        println!("Goal: ({}, {})", goal.0, goal.1);

        Self { maze, player_start }
    }
}

struct FpsCounter {
    frame_count: u32,
    elapsed: f32,
    displayed_fps: u32,
}

impl FpsCounter {
    fn new() -> Self {
        Self {
            frame_count: 0,
            elapsed: 0.0,
            displayed_fps: 0,
        }
    }

    fn update(&mut self, delta_time: f32) {
        self.frame_count += 1;
        self.elapsed += delta_time;

        if self.elapsed >= FPS_UPDATE_INTERVAL {
            self.displayed_fps = (self.frame_count as f32 / self.elapsed).round() as u32;
            self.frame_count = 0;
            self.elapsed = 0.0;
        }
    }

    fn fps(&self) -> u32 {
        self.displayed_fps
    }
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WelcomeMenuOption {
    Start,
    ChangeLevel,
}

impl WelcomeMenuOption {
    const OPTIONS: [Self; 2] = [Self::Start, Self::ChangeLevel];

    fn index(self) -> usize {
        match self {
            Self::Start => 0,
            Self::ChangeLevel => 1,
        }
    }

    fn from_index(index: usize) -> Self {
        Self::OPTIONS[index % Self::OPTIONS.len()]
    }
}

fn load_levels(paths: &[&str]) -> Vec<Level> {
    paths.iter().map(|path| Level::load(path)).collect()
}

fn next_level_index(current: usize, level_count: usize) -> usize {
    if level_count == 0 {
        0
    } else {
        (current + 1) % level_count
    }
}

fn previous_level_index(current: usize, level_count: usize) -> usize {
    if level_count == 0 {
        0
    } else {
        (current + level_count - 1) % level_count
    }
}

fn next_menu_index(current: usize, option_count: usize) -> usize {
    if option_count == 0 {
        0
    } else {
        (current + 1) % option_count
    }
}

fn previous_menu_index(current: usize, option_count: usize) -> usize {
    if option_count == 0 {
        0
    } else {
        (current + option_count - 1) % option_count
    }
}

fn main() -> Result<(), minifb::Error> {
    let levels = load_levels(&LEVEL_PATHS);
    let mut selected_level_index = 0;
    let mut player = Player::new(
        levels[selected_level_index].player_start.0,
        levels[selected_level_index].player_start.1,
        BLOCK_SIZE,
    );

    println!(
        "Player world position: ({:.1}, {:.1})",
        player.pos.x, player.pos.y
    );
    println!("Player angle: {:.4} rad", player.a);
    println!("Player FOV: {:.4} rad", player.fov);

    let textures = TextureManager::load_default();

    let mut window = Window::new(
        "Proyecto 1 - Raycasting",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);
    let mut last_time = Instant::now();
    let mut render_mode = RenderMode::Mode3D;
    let mut game_state = GameState::Welcome;
    let mut welcome_menu_option = WelcomeMenuOption::Start;
    let mut fps_counter = FpsCounter::new();
    let mut mouse_look = MouseLook::new();
    let mut gamepad_input = GamepadInput::new();

    while window.is_open() && !window.is_key_down(Key::Escape) {
        let current_time = Instant::now();
        let delta_time = current_time
            .duration_since(last_time)
            .as_secs_f32()
            .min(0.1);
        last_time = current_time;
        fps_counter.update(delta_time);
        let gamepad = gamepad_input.update();

        match game_state {
            GameState::Welcome => {
                let previous_level = selected_level_index;

                if window.is_key_pressed(Key::W, KeyRepeat::No)
                    || window.is_key_pressed(Key::Up, KeyRepeat::No)
                    || gamepad.menu_up_pressed()
                {
                    welcome_menu_option =
                        WelcomeMenuOption::from_index(previous_menu_index(
                            welcome_menu_option.index(),
                            WelcomeMenuOption::OPTIONS.len(),
                        ));
                }

                if window.is_key_pressed(Key::S, KeyRepeat::No)
                    || window.is_key_pressed(Key::Down, KeyRepeat::No)
                    || gamepad.menu_down_pressed()
                {
                    welcome_menu_option = WelcomeMenuOption::from_index(next_menu_index(
                        welcome_menu_option.index(),
                        WelcomeMenuOption::OPTIONS.len(),
                    ));
                }

                if window.is_key_pressed(Key::A, KeyRepeat::No)
                    || window.is_key_pressed(Key::Left, KeyRepeat::No)
                    || gamepad.previous_level_pressed()
                {
                    selected_level_index = previous_level_index(selected_level_index, levels.len());
                }

                if window.is_key_pressed(Key::D, KeyRepeat::No)
                    || window.is_key_pressed(Key::Right, KeyRepeat::No)
                    || gamepad.next_level_pressed()
                {
                    selected_level_index = next_level_index(selected_level_index, levels.len());
                }

                if selected_level_index != previous_level {
                    reset_player(
                        &mut player,
                        levels[selected_level_index].player_start,
                        BLOCK_SIZE,
                    );
                    render_mode = RenderMode::Mode3D;
                }

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) || gamepad.confirm_pressed() {
                    match welcome_menu_option {
                        WelcomeMenuOption::Start => {
                            reset_player(
                                &mut player,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                            game_state = GameState::Playing;
                        }
                        WelcomeMenuOption::ChangeLevel => {
                            selected_level_index =
                                next_level_index(selected_level_index, levels.len());
                            reset_player(
                                &mut player,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                        }
                    }
                }

                mouse_look.reset();
            }
            GameState::Playing => {
                let current_level = &levels[selected_level_index];

                process_input(
                    &window,
                    &mut player,
                    &mut mouse_look,
                    &current_level.maze,
                    BLOCK_SIZE,
                    delta_time,
                    &gamepad,
                );

                if player_reached_goal(&current_level.maze, &player, BLOCK_SIZE) {
                    game_state = GameState::Won;
                } else if window.is_key_pressed(Key::Tab, KeyRepeat::No)
                    || gamepad.toggle_view_pressed()
                {
                    render_mode = render_mode.toggle();
                }
            }
            GameState::Won => {
                if window.is_key_pressed(Key::R, KeyRepeat::No) || gamepad.restart_pressed() {
                    reset_player(
                        &mut player,
                        levels[selected_level_index].player_start,
                        BLOCK_SIZE,
                    );
                    game_state = GameState::Playing;
                }

                mouse_look.reset();
            }
        }

        framebuffer.clear();
        let current_level = &levels[selected_level_index];

        match game_state {
            GameState::Welcome => {
                render_welcome_screen(
                    &mut framebuffer,
                    &current_level.maze,
                    selected_level_index,
                    levels.len(),
                    welcome_menu_option.index(),
                );
            }
            GameState::Playing => match render_mode {
                RenderMode::Mode2D => {
                    let (maze_offset_x, maze_offset_y) =
                        maze_offset(&framebuffer, &current_level.maze, BLOCK_SIZE);

                    render_maze(&mut framebuffer, &current_level.maze, BLOCK_SIZE);
                    cast_fov_2d(
                        &mut framebuffer,
                        &current_level.maze,
                        &player,
                        BLOCK_SIZE,
                        maze_offset_x,
                        maze_offset_y,
                    );
                    render_player(&mut framebuffer, &player, maze_offset_x, maze_offset_y);
                }
                RenderMode::Mode3D => {
                    render_3d(
                        &mut framebuffer,
                        &current_level.maze,
                        &player,
                        BLOCK_SIZE,
                        &textures,
                    );
                    render_minimap(&mut framebuffer, &current_level.maze, &player, BLOCK_SIZE);
                }
            },
            GameState::Won => {
                render_victory_screen(&mut framebuffer, &current_level.maze);
            }
        }

        if game_state == GameState::Playing {
            render_fps_overlay(&mut framebuffer, fps_counter.fps());
        }

        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fps_counter_starts_at_zero() {
        let fps_counter = FpsCounter::new();

        assert_eq!(fps_counter.fps(), 0);
    }

    #[test]
    fn fps_counter_updates_after_interval() {
        let mut fps_counter = FpsCounter::new();

        for _ in 0..30 {
            fps_counter.update(1.0 / 60.0);
        }

        assert_eq!(fps_counter.fps(), 60);
    }

    #[test]
    fn next_level_index_wraps_to_first_level() {
        assert_eq!(next_level_index(0, 3), 1);
        assert_eq!(next_level_index(2, 3), 0);
    }

    #[test]
    fn previous_level_index_wraps_to_last_level() {
        assert_eq!(previous_level_index(2, 3), 1);
        assert_eq!(previous_level_index(0, 3), 2);
    }

    #[test]
    fn level_index_helpers_accept_empty_level_lists() {
        assert_eq!(next_level_index(0, 0), 0);
        assert_eq!(previous_level_index(0, 0), 0);
    }

    #[test]
    fn menu_index_helpers_wrap_selection() {
        assert_eq!(next_menu_index(0, 2), 1);
        assert_eq!(next_menu_index(1, 2), 0);
        assert_eq!(previous_menu_index(1, 2), 0);
        assert_eq!(previous_menu_index(0, 2), 1);
    }
}
