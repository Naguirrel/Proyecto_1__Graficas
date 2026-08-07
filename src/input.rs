use std::f32::consts::PI;

use minifb::{Key, Window};

use crate::player::Player;

const MOVEMENT_SPEED: f32 = 100.0;
const ROTATION_SPEED: f32 = 2.0;

pub fn process_input(window: &Window, player: &mut Player, delta_time: f32) {
    let dir_x = player.a.cos();
    let dir_y = player.a.sin();
    let movement = MOVEMENT_SPEED * delta_time;
    let rotation = ROTATION_SPEED * delta_time;

    if window.is_key_down(Key::W) {
        player.pos.x += dir_x * movement;
        player.pos.y += dir_y * movement;
    }

    if window.is_key_down(Key::S) {
        player.pos.x -= dir_x * movement;
        player.pos.y -= dir_y * movement;
    }

    if window.is_key_down(Key::A) {
        player.a -= rotation;
    }

    if window.is_key_down(Key::D) {
        player.a += rotation;
    }

    player.a = player.a.rem_euclid(2.0 * PI);
}
