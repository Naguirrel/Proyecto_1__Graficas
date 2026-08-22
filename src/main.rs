mod audio;
mod caster;
mod framebuffer;
mod game;
mod gamepad;
mod input;
mod line;
mod maze;
mod player;
mod render;
mod sprite;
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
    maze_offset, render_3d_with_sprites, render_fps_overlay, render_loss_screen, render_maze,
    render_minimap, render_pause_menu, render_player, render_victory_screen, render_welcome_screen,
};
use sprite::{SpriteState, SpriteUpdate};
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
    Exit,
}

impl WelcomeMenuOption {
    const OPTIONS: [Self; 3] = [Self::Start, Self::ChangeLevel, Self::Exit];

    fn index(self) -> usize {
        match self {
            Self::Start => 0,
            Self::ChangeLevel => 1,
            Self::Exit => 2,
        }
    }

    fn from_index(index: usize) -> Self {
        Self::OPTIONS[index % Self::OPTIONS.len()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum VictoryMenuOption {
    Restart,
    NextLevel,
    MainMenu,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum PauseMenuOption {
    Continue,
    ChangeLevel,
    MainMenu,
}

impl PauseMenuOption {
    const OPTIONS: [Self; 3] = [Self::Continue, Self::ChangeLevel, Self::MainMenu];

    fn at(index: usize) -> Self {
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

fn has_next_level(current: usize, level_count: usize) -> bool {
    current + 1 < level_count
}

fn victory_option_count(current_level: usize, level_count: usize) -> usize {
    if has_next_level(current_level, level_count) {
        3
    } else {
        2
    }
}

fn victory_option_at(index: usize, current_level: usize, level_count: usize) -> VictoryMenuOption {
    match (index, has_next_level(current_level, level_count)) {
        (0, _) => VictoryMenuOption::Restart,
        (1, true) => VictoryMenuOption::NextLevel,
        _ => VictoryMenuOption::MainMenu,
    }
}

fn reset_level_state(
    player: &mut Player,
    sprite_state: &mut SpriteState,
    level_index: usize,
    player_start: (usize, usize),
    block_size: usize,
) {
    reset_player(player, player_start, block_size);
    sprite_state.reset_for_level(level_index, block_size);
}

fn main() -> Result<(), minifb::Error> {
    let levels = load_levels(&LEVEL_PATHS);
    let mut selected_level_index = 0;
    let mut player = Player::new(
        levels[selected_level_index].player_start.0,
        levels[selected_level_index].player_start.1,
        BLOCK_SIZE,
    );
    let mut sprite_state = SpriteState::for_level(selected_level_index, BLOCK_SIZE);

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
    let mut victory_menu_index = 0;
    let mut pause_menu_index = 0;
    let mut pause_level_index = selected_level_index;
    let mut fps_counter = FpsCounter::new();
    let mut mouse_look = MouseLook::new();
    let mut gamepad_input = GamepadInput::new();
    let mut should_exit = false;

    while window.is_open() && !window.is_key_down(Key::Escape) && !should_exit {
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
                    welcome_menu_option = WelcomeMenuOption::from_index(previous_menu_index(
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
                    reset_level_state(
                        &mut player,
                        &mut sprite_state,
                        selected_level_index,
                        levels[selected_level_index].player_start,
                        BLOCK_SIZE,
                    );
                    render_mode = RenderMode::Mode3D;
                }

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) || gamepad.confirm_pressed() {
                    match welcome_menu_option {
                        WelcomeMenuOption::Start => {
                            reset_level_state(
                                &mut player,
                                &mut sprite_state,
                                selected_level_index,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                            game_state = GameState::Playing;
                        }
                        WelcomeMenuOption::ChangeLevel => {
                            selected_level_index =
                                next_level_index(selected_level_index, levels.len());
                            reset_level_state(
                                &mut player,
                                &mut sprite_state,
                                selected_level_index,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                        }
                        WelcomeMenuOption::Exit => {
                            should_exit = true;
                        }
                    }
                }

                mouse_look.reset();
            }
            GameState::Playing => {
                let current_level = &levels[selected_level_index];

                if window.is_key_pressed(Key::P, KeyRepeat::No) || gamepad.pause_pressed() {
                    pause_menu_index = 0;
                    pause_level_index = selected_level_index;
                    game_state = GameState::Paused;
                    mouse_look.reset();
                } else {
                    process_input(
                        &window,
                        &mut player,
                        &mut mouse_look,
                        &current_level.maze,
                        BLOCK_SIZE,
                        delta_time,
                        &gamepad,
                    );
                    if sprite_state.update(&player, &current_level.maze, BLOCK_SIZE, delta_time)
                        == SpriteUpdate::PlayerCaught
                    {
                        game_state = GameState::Lost;
                        mouse_look.reset();
                    }

                    if game_state == GameState::Playing
                        && player_reached_goal(&current_level.maze, &player, BLOCK_SIZE)
                    {
                        game_state = GameState::Won;
                        victory_menu_index = 0;
                    } else if game_state == GameState::Playing
                        && (window.is_key_pressed(Key::Tab, KeyRepeat::No)
                            || gamepad.toggle_view_pressed())
                    {
                        render_mode = render_mode.toggle();
                    }
                }
            }
            GameState::Paused => {
                if window.is_key_pressed(Key::P, KeyRepeat::No) || gamepad.pause_pressed() {
                    game_state = GameState::Playing;
                } else {
                    if window.is_key_pressed(Key::W, KeyRepeat::No)
                        || window.is_key_pressed(Key::Up, KeyRepeat::No)
                        || gamepad.menu_up_pressed()
                    {
                        pause_menu_index =
                            previous_menu_index(pause_menu_index, PauseMenuOption::OPTIONS.len());
                    }

                    if window.is_key_pressed(Key::S, KeyRepeat::No)
                        || window.is_key_pressed(Key::Down, KeyRepeat::No)
                        || gamepad.menu_down_pressed()
                    {
                        pause_menu_index =
                            next_menu_index(pause_menu_index, PauseMenuOption::OPTIONS.len());
                    }

                    if PauseMenuOption::at(pause_menu_index) == PauseMenuOption::ChangeLevel {
                        if window.is_key_pressed(Key::A, KeyRepeat::No)
                            || window.is_key_pressed(Key::Left, KeyRepeat::No)
                            || gamepad.previous_level_pressed()
                        {
                            pause_level_index =
                                previous_level_index(pause_level_index, levels.len());
                        }

                        if window.is_key_pressed(Key::D, KeyRepeat::No)
                            || window.is_key_pressed(Key::Right, KeyRepeat::No)
                            || gamepad.next_level_pressed()
                        {
                            pause_level_index = next_level_index(pause_level_index, levels.len());
                        }
                    }

                    if window.is_key_pressed(Key::Enter, KeyRepeat::No) || gamepad.confirm_pressed()
                    {
                        match PauseMenuOption::at(pause_menu_index) {
                            PauseMenuOption::Continue => {
                                game_state = GameState::Playing;
                            }
                            PauseMenuOption::ChangeLevel => {
                                selected_level_index = pause_level_index;
                                reset_level_state(
                                    &mut player,
                                    &mut sprite_state,
                                    selected_level_index,
                                    levels[selected_level_index].player_start,
                                    BLOCK_SIZE,
                                );
                                render_mode = RenderMode::Mode3D;
                                game_state = GameState::Playing;
                            }
                            PauseMenuOption::MainMenu => {
                                reset_level_state(
                                    &mut player,
                                    &mut sprite_state,
                                    selected_level_index,
                                    levels[selected_level_index].player_start,
                                    BLOCK_SIZE,
                                );
                                render_mode = RenderMode::Mode3D;
                                game_state = GameState::Welcome;
                            }
                        }
                    }
                }

                mouse_look.reset();
            }
            GameState::Won => {
                let option_count = victory_option_count(selected_level_index, levels.len());

                if window.is_key_pressed(Key::W, KeyRepeat::No)
                    || window.is_key_pressed(Key::Up, KeyRepeat::No)
                    || gamepad.menu_up_pressed()
                {
                    victory_menu_index = previous_menu_index(victory_menu_index, option_count);
                }

                if window.is_key_pressed(Key::S, KeyRepeat::No)
                    || window.is_key_pressed(Key::Down, KeyRepeat::No)
                    || gamepad.menu_down_pressed()
                {
                    victory_menu_index = next_menu_index(victory_menu_index, option_count);
                }

                if window.is_key_pressed(Key::R, KeyRepeat::No) {
                    reset_level_state(
                        &mut player,
                        &mut sprite_state,
                        selected_level_index,
                        levels[selected_level_index].player_start,
                        BLOCK_SIZE,
                    );
                    game_state = GameState::Playing;
                }

                if window.is_key_pressed(Key::Enter, KeyRepeat::No) || gamepad.confirm_pressed() {
                    match victory_option_at(victory_menu_index, selected_level_index, levels.len())
                    {
                        VictoryMenuOption::Restart => {
                            reset_level_state(
                                &mut player,
                                &mut sprite_state,
                                selected_level_index,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                            game_state = GameState::Playing;
                        }
                        VictoryMenuOption::NextLevel => {
                            selected_level_index += 1;
                            reset_level_state(
                                &mut player,
                                &mut sprite_state,
                                selected_level_index,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                            render_mode = RenderMode::Mode3D;
                            game_state = GameState::Playing;
                        }
                        VictoryMenuOption::MainMenu => {
                            reset_level_state(
                                &mut player,
                                &mut sprite_state,
                                selected_level_index,
                                levels[selected_level_index].player_start,
                                BLOCK_SIZE,
                            );
                            render_mode = RenderMode::Mode3D;
                            game_state = GameState::Welcome;
                        }
                    }
                }

                mouse_look.reset();
            }
            GameState::Lost => {
                if window.is_key_pressed(Key::R, KeyRepeat::No)
                    || window.is_key_pressed(Key::Enter, KeyRepeat::No)
                    || gamepad.confirm_pressed()
                {
                    reset_level_state(
                        &mut player,
                        &mut sprite_state,
                        selected_level_index,
                        levels[selected_level_index].player_start,
                        BLOCK_SIZE,
                    );
                    render_mode = RenderMode::Mode3D;
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
                    render_3d_with_sprites(
                        &mut framebuffer,
                        &current_level.maze,
                        &player,
                        BLOCK_SIZE,
                        &textures,
                        &sprite_state.sprites,
                        sprite_state.has_food_power(),
                    );
                    render_minimap(&mut framebuffer, &current_level.maze, &player, BLOCK_SIZE);
                }
            },
            GameState::Paused => {
                render_pause_menu(
                    &mut framebuffer,
                    &levels[pause_level_index].maze,
                    pause_level_index,
                    levels.len(),
                    pause_menu_index,
                );
            }
            GameState::Won => {
                render_victory_screen(
                    &mut framebuffer,
                    &current_level.maze,
                    selected_level_index,
                    levels.len(),
                    victory_menu_index,
                );
            }
            GameState::Lost => {
                render_loss_screen(
                    &mut framebuffer,
                    &current_level.maze,
                    selected_level_index,
                    levels.len(),
                );
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

    #[test]
    fn victory_menu_hides_next_level_on_last_level() {
        assert_eq!(victory_option_count(0, 3), 3);
        assert_eq!(victory_option_count(2, 3), 2);
        assert_eq!(victory_option_at(1, 0, 3), VictoryMenuOption::NextLevel);
        assert_eq!(victory_option_at(1, 2, 3), VictoryMenuOption::MainMenu);
    }

    #[test]
    fn pause_menu_options_keep_fixed_order() {
        assert_eq!(PauseMenuOption::at(0), PauseMenuOption::Continue);
        assert_eq!(PauseMenuOption::at(1), PauseMenuOption::ChangeLevel);
        assert_eq!(PauseMenuOption::at(2), PauseMenuOption::MainMenu);
    }

    #[test]
    fn reset_level_state_resets_player_and_sprites() {
        let mut player = Player::new(7, 5, BLOCK_SIZE);
        let mut sprite_state = SpriteState::for_level(0, BLOCK_SIZE);
        sprite_state.food_power_timer = 5.0;
        sprite_state.sprites[0].active = false;

        reset_level_state(&mut player, &mut sprite_state, 0, (1, 1), BLOCK_SIZE);

        assert_eq!(player.pos.x, 60.0);
        assert_eq!(player.pos.y, 60.0);
        assert_eq!(sprite_state.food_power_timer, 0.0);
        assert!(sprite_state.sprites.iter().all(|sprite| sprite.active));
    }
}
