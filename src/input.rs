use std::f32::consts::PI;
use std::fs::File;
use std::io::{self, Read};
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver};
use std::thread;

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
const CONTROLLER_DEADZONE: f32 = 0.2;
const CONTROLLER_LOOK_DEADZONE: f32 = 0.35;
const CONTROLLER_AXIS_COUNT: usize = 16;
const CONTROLLER_BUTTON_COUNT: usize = 32;
const JS_EVENT_BUTTON: u8 = 0x01;
const JS_EVENT_AXIS: u8 = 0x02;
const JS_EVENT_INIT: u8 = 0x80;
const LEFT_STICK_Y_AXIS: usize = 1;
const RIGHT_STICK_X_AXIS: usize = 3;
const DPAD_X_AXIS: usize = 6;
const DPAD_Y_AXIS: usize = 7;
const SOUTH_BUTTON: usize = 0;
const SELECT_BUTTON: usize = 6;
const START_BUTTON: usize = 7;
const DPAD_UP_BUTTONS: [usize; 1] = [11];
const DPAD_DOWN_BUTTONS: [usize; 1] = [12];
const DPAD_LEFT_BUTTONS: [usize; 1] = [13];
const DPAD_RIGHT_BUTTONS: [usize; 1] = [14];

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

#[derive(Default)]
struct ControllerFrame {
    start_pressed: bool,
    select_pressed: bool,
    south_pressed: bool,
    dpad_left_pressed: bool,
    dpad_right_pressed: bool,
}

pub struct ControllerInput {
    receiver: Option<Receiver<ControllerEvent>>,
    axes: [f32; CONTROLLER_AXIS_COUNT],
    buttons: [bool; CONTROLLER_BUTTON_COUNT],
    frame: ControllerFrame,
}

impl ControllerInput {
    pub fn new() -> Self {
        let receiver = match open_controller_device() {
            Ok(Some((path, file))) => {
                println!("Controller input enabled: {}", path.display());
                Some(spawn_controller_reader(file))
            }
            Ok(None) => {
                eprintln!("Controller input disabled: no /dev/input/js* device found");
                None
            }
            Err(error) => {
                eprintln!("Controller input disabled: {error}");
                None
            }
        };

        Self {
            receiver,
            axes: [0.0; CONTROLLER_AXIS_COUNT],
            buttons: [false; CONTROLLER_BUTTON_COUNT],
            frame: ControllerFrame::default(),
        }
    }

    pub fn update(&mut self) {
        self.frame = ControllerFrame::default();

        let Some(receiver) = &self.receiver else {
            return;
        };

        while let Ok(event) = receiver.try_recv() {
            match event {
                ControllerEvent::Button { number, pressed } => {
                    if number < self.buttons.len() {
                        self.buttons[number] = pressed;
                    }

                    if pressed {
                        self.frame.start_pressed |= number == START_BUTTON;
                        self.frame.select_pressed |= number == SELECT_BUTTON;
                        self.frame.south_pressed |= number == SOUTH_BUTTON;
                        self.frame.dpad_left_pressed |= DPAD_LEFT_BUTTONS.contains(&number);
                        self.frame.dpad_right_pressed |= DPAD_RIGHT_BUTTONS.contains(&number);
                    }
                }
                ControllerEvent::Axis { number, value } => {
                    if number >= self.axes.len() {
                        continue;
                    }

                    let previous = self.axes[number];
                    self.axes[number] = value;

                    if number == DPAD_X_AXIS {
                        self.frame.dpad_left_pressed |=
                            previous >= -CONTROLLER_DEADZONE && value < -CONTROLLER_DEADZONE;
                        self.frame.dpad_right_pressed |=
                            previous <= CONTROLLER_DEADZONE && value > CONTROLLER_DEADZONE;
                    }
                }
            }
        }
    }

    pub fn menu_left_pressed(&self) -> bool {
        self.frame.dpad_left_pressed
    }

    pub fn menu_right_pressed(&self) -> bool {
        self.frame.dpad_right_pressed
    }

    pub fn start_pressed(&self) -> bool {
        self.frame.start_pressed || self.frame.south_pressed
    }

    pub fn select_pressed(&self) -> bool {
        self.frame.select_pressed
    }

    pub fn jump_pressed(&self) -> bool {
        self.frame.south_pressed
    }

    fn movement_axis(&self) -> f32 {
        nonzero_axis(-self.axis(LEFT_STICK_Y_AXIS))
            .or_else(|| nonzero_axis(-self.axis(DPAD_Y_AXIS)))
            .or_else(|| self.button_axis(&DPAD_UP_BUTTONS, &DPAD_DOWN_BUTTONS))
            .unwrap_or(0.0)
    }

    fn rotation_axis(&self) -> f32 {
        nonzero_axis_with_deadzone(self.axis(RIGHT_STICK_X_AXIS), CONTROLLER_LOOK_DEADZONE)
            .or_else(|| nonzero_axis_with_deadzone(self.axis(DPAD_X_AXIS), CONTROLLER_DEADZONE))
            .or_else(|| self.button_axis(&DPAD_RIGHT_BUTTONS, &DPAD_LEFT_BUTTONS))
            .unwrap_or(0.0)
    }

    fn axis(&self, number: usize) -> f32 {
        self.axes.get(number).copied().unwrap_or(0.0)
    }

    fn button_axis(&self, positive_buttons: &[usize], negative_buttons: &[usize]) -> Option<f32> {
        let mut value = 0.0;

        if positive_buttons.iter().any(|button| self.button(*button)) {
            value += 1.0;
        }

        if negative_buttons.iter().any(|button| self.button(*button)) {
            value -= 1.0;
        }

        (value != 0.0).then_some(value)
    }

    fn button(&self, number: usize) -> bool {
        self.buttons.get(number).copied().unwrap_or(false)
    }
}

pub fn process_input(
    window: &Window,
    player: &mut Player,
    mouse_look: &mut MouseLook,
    controller: &ControllerInput,
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

    move_direction += controller.movement_axis();
    move_direction = move_direction.clamp(-1.0, 1.0);

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

    player.a += controller.rotation_axis() * rotation;

    process_mouse_look(window, player, mouse_look);

    player.a = player.a.rem_euclid(2.0 * PI);

    if window.is_key_pressed(Key::Space, KeyRepeat::No) || gamepad.jump_pressed() {
    if window.is_key_pressed(Key::Space, KeyRepeat::No) || controller.jump_pressed() {
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

fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone { 0.0 } else { value }
}

fn nonzero_axis(value: f32) -> Option<f32> {
    nonzero_axis_with_deadzone(value, CONTROLLER_DEADZONE)
}

fn nonzero_axis_with_deadzone(value: f32, deadzone: f32) -> Option<f32> {
    let value = apply_deadzone(value, deadzone);

    (value != 0.0).then_some(value)
}

enum ControllerEvent {
    Axis { number: usize, value: f32 },
    Button { number: usize, pressed: bool },
}

fn open_controller_device() -> io::Result<Option<(PathBuf, File)>> {
    let mut first_error = None;

    for index in 0..4 {
        let path = PathBuf::from(format!("/dev/input/js{index}"));

        match File::open(&path) {
            Ok(file) => return Ok(Some((path, file))),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => {
                first_error.get_or_insert_with(|| format!("{}: {error}", path.display()));
            }
        }
    }

    if let Some(error) = first_error {
        Err(io::Error::new(io::ErrorKind::PermissionDenied, error))
    } else {
        Ok(None)
    }
}

fn spawn_controller_reader(mut file: File) -> Receiver<ControllerEvent> {
    let (sender, receiver) = mpsc::channel();

    thread::spawn(move || {
        loop {
            let mut buffer = [0_u8; 8];

            if file.read_exact(&mut buffer).is_err() {
                break;
            }

            let event_type = buffer[6] & !JS_EVENT_INIT;
            let number = buffer[7] as usize;

            let event = match event_type {
                JS_EVENT_BUTTON => Some(ControllerEvent::Button {
                    number,
                    pressed: buffer[4] != 0 || buffer[5] != 0,
                }),
                JS_EVENT_AXIS => Some(ControllerEvent::Axis {
                    number,
                    value: normalize_axis(i16::from_ne_bytes([buffer[4], buffer[5]])),
                }),
                _ => None,
            };

            if let Some(event) = event
                && sender.send(event).is_err()
            {
                break;
            }
        }
    });

    receiver
}

fn normalize_axis(value: i16) -> f32 {
    (value as f32 / i16::MAX as f32).clamp(-1.0, 1.0)
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

    #[test]
    fn controller_deadzone_ignores_small_axis_values() {
        assert_eq!(apply_deadzone(0.19, CONTROLLER_DEADZONE), 0.0);
        assert_eq!(apply_deadzone(-0.19, CONTROLLER_DEADZONE), 0.0);
    }

    #[test]
    fn controller_deadzone_keeps_large_axis_values() {
        assert_eq!(apply_deadzone(0.5, CONTROLLER_DEADZONE), 0.5);
        assert_eq!(apply_deadzone(-0.5, CONTROLLER_DEADZONE), -0.5);
    }

    #[test]
    fn normalize_axis_maps_joystick_range_to_unit_range() {
        assert_eq!(normalize_axis(0), 0.0);
        assert_eq!(normalize_axis(i16::MAX), 1.0);
        assert_eq!(normalize_axis(i16::MIN), -1.0);
    }
}
