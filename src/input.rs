use std::f32::consts::PI;

use minifb::{Key, KeyRepeat, Window};

use crate::maze::{Maze, is_walkable};
use crate::player::Player;

const MOVEMENT_SPEED: f32 = 100.0;
const ROTATION_SPEED: f32 = 2.0;
const JUMP_SPEED: f32 = 220.0;
const GRAVITY: f32 = 600.0;

pub fn process_input(
    window: &Window,
    player: &mut Player,
    maze: &Maze,
    block_size: usize,
    delta_time: f32,
) {
    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let movement = MOVEMENT_SPEED * delta_time;
    let rotation = ROTATION_SPEED * delta_time;

    let mut move_direction = 0.0;

    if window.is_key_down(Key::W) {
        move_direction += 1.0;
    }

    if window.is_key_down(Key::S) {
        move_direction -= 1.0;
    }

    if move_direction != 0.0 {
        let candidate_x = player.pos.x + dir_x * movement * move_direction;
        let candidate_y = player.pos.y + dir_y * movement * move_direction;

        if is_walkable(maze, candidate_x, player.pos.y, block_size) {
            player.pos.x = candidate_x;
        }

        if is_walkable(maze, player.pos.x, candidate_y, block_size) {
            player.pos.y = candidate_y;
        }
    }

    if window.is_key_down(Key::A) {
        player.a -= rotation;
    }

    if window.is_key_down(Key::D) {
        player.a += rotation;
    }

    player.a = player.a.rem_euclid(2.0 * PI);

    if window.is_key_pressed(Key::Space, KeyRepeat::No) {
        player.start_jump(JUMP_SPEED);
    }

    player.update_vertical_motion(delta_time, GRAVITY);
}
