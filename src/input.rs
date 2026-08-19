use std::f32::consts::PI;

use minifb::{Key, KeyRepeat, MouseMode, Window};

use crate::gamepad::GamepadSnapshot;
use crate::maze::{Maze, is_walkable};
use crate::player::Player;

const MOVEMENT_SPEED: f32 = 100.0;
const ROTATION_SPEED: f32 = 2.0;
const MOUSE_SENSITIVITY: f32 = 0.003;
const MAX_MOUSE_DELTA: f32 = 100.0;
const JUMP_SPEED: f32 = 220.0;
const GRAVITY: f32 = 600.0;

pub struct MouseLook {
    previous_x: Option<f32>,
}

impl MouseLook {
    pub fn new() -> Self {
        Self { previous_x: None }
    }

    pub fn reset(&mut self) {
        self.previous_x = None;
    }
}

pub fn process_input(
    window: &Window,
    player: &mut Player,
    mouse_look: &mut MouseLook,
    maze: &Maze,
    block_size: usize,
    delta_time: f32,
    gamepad: &GamepadSnapshot,
) {
    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let movement = MOVEMENT_SPEED * delta_time;
    let rotation = ROTATION_SPEED * delta_time;

    let move_direction = keyboard_movement_axis(window) + gamepad.movement_axis();

    if move_direction != 0.0 {
        let move_direction = move_direction.clamp(-1.0, 1.0);
        let candidate_x = player.pos.x + dir_x * movement * move_direction;
        let candidate_y = player.pos.y + dir_y * movement * move_direction;

        if is_walkable(maze, candidate_x, player.pos.y, block_size) {
            player.pos.x = candidate_x;
        }

        if is_walkable(maze, player.pos.x, candidate_y, block_size) {
            player.pos.y = candidate_y;
        }
    }

    let rotation_direction = keyboard_rotation_axis(window) + gamepad.rotation_axis();
    if rotation_direction != 0.0 {
        player.a += rotation * rotation_direction.clamp(-1.0, 1.0);
    }

    process_mouse_look(window, player, mouse_look);

    player.a = player.a.rem_euclid(2.0 * PI);

    if window.is_key_pressed(Key::Space, KeyRepeat::No) || gamepad.jump_pressed() {
        player.start_jump(JUMP_SPEED);
    }

    player.update_vertical_motion(delta_time, GRAVITY);
}

fn keyboard_movement_axis(window: &Window) -> f32 {
    let mut move_direction = 0.0;

    if window.is_key_down(Key::W) {
        move_direction += 1.0;
    }

    if window.is_key_down(Key::S) {
        move_direction -= 1.0;
    }

    move_direction
}

fn keyboard_rotation_axis(window: &Window) -> f32 {
    let mut rotation_direction = 0.0;

    if window.is_key_down(Key::A) {
        rotation_direction -= 1.0;
    }

    if window.is_key_down(Key::D) {
        rotation_direction += 1.0;
    }

    rotation_direction
}

fn process_mouse_look(window: &Window, player: &mut Player, mouse_look: &mut MouseLook) {
    let Some((current_x, _)) = window.get_mouse_pos(MouseMode::Discard) else {
        mouse_look.reset();
        return;
    };

    let Some(previous_x) = mouse_look.previous_x.replace(current_x) else {
        return;
    };

    let delta_x = (current_x - previous_x).clamp(-MAX_MOUSE_DELTA, MAX_MOUSE_DELTA);
    player.a += mouse_delta_to_rotation(delta_x);
}

fn mouse_delta_to_rotation(delta_x: f32) -> f32 {
    delta_x * MOUSE_SENSITIVITY
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mouse_delta_to_rotation_is_positive_when_mouse_moves_right() {
        assert!(mouse_delta_to_rotation(10.0) > 0.0);
    }

    #[test]
    fn mouse_delta_to_rotation_is_negative_when_mouse_moves_left() {
        assert!(mouse_delta_to_rotation(-10.0) < 0.0);
    }

    #[test]
    fn mouse_delta_to_rotation_is_zero_without_mouse_movement() {
        assert_eq!(mouse_delta_to_rotation(0.0), 0.0);
    }

    #[test]
    fn mouse_delta_to_rotation_scales_with_delta() {
        let small_rotation = mouse_delta_to_rotation(5.0);
        let large_rotation = mouse_delta_to_rotation(10.0);

        assert!(large_rotation > small_rotation);
    }
}
