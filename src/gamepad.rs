use gilrs::{Axis, Button, Event, EventType, GamepadId, Gilrs};

const STICK_DEADZONE: f32 = 0.18;
const TRIGGER_DEADZONE: f32 = 0.35;

#[derive(Debug, Clone, Copy, Default)]
struct GamepadButtons {
    south: bool,
    east: bool,
    north: bool,
    west: bool,
    select: bool,
    start: bool,
    dpad_left: bool,
    dpad_right: bool,
    dpad_up: bool,
    dpad_down: bool,
    left_trigger: bool,
    right_trigger: bool,
    right_thumb: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GamepadSnapshot {
    left_stick_x: f32,
    left_stick_y: f32,
    right_stick_x: f32,
    previous_left_stick_x: f32,
    previous_left_stick_y: f32,
    previous_buttons: GamepadButtons,
    buttons: GamepadButtons,
}

impl GamepadSnapshot {
    pub fn movement_axis(&self) -> f32 {
        let stick = apply_deadzone(self.left_stick_y, STICK_DEADZONE);
        let digital = digital_axis(self.buttons.dpad_up, self.buttons.dpad_down);

        strongest_axis(stick, digital)
    }

    pub fn rotation_axis(&self) -> f32 {
        let stick = apply_deadzone(self.right_stick_x, STICK_DEADZONE);
        let shoulder_buttons = self.buttons.left_trigger || self.buttons.right_trigger;
        let shoulder = digital_axis(self.buttons.right_trigger, self.buttons.left_trigger);
        let dpad = digital_axis(self.buttons.dpad_right, self.buttons.dpad_left);

        if shoulder_buttons {
            strongest_axis(stick, shoulder)
        } else {
            strongest_axis(stick, dpad)
        }
    }

    pub fn jump_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.south)
    }

    pub fn confirm_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.south || buttons.start)
    }

    pub fn restart_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.south || buttons.start)
    }

    pub fn toggle_view_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.north || buttons.select || buttons.right_thumb)
    }

    pub fn previous_level_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.dpad_left || buttons.west || buttons.left_trigger)
            || self.axis_pressed_left()
    }

    pub fn next_level_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.dpad_right || buttons.east || buttons.right_trigger)
            || self.axis_pressed_right()
    }

    pub fn menu_up_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.dpad_up) || self.axis_pressed_up()
    }

    pub fn menu_down_pressed(&self) -> bool {
        self.button_pressed(|buttons| buttons.dpad_down) || self.axis_pressed_down()
    }

    fn button_pressed<F>(&self, button_selector: F) -> bool
    where
        F: Fn(GamepadButtons) -> bool,
    {
        button_selector(self.buttons) && !button_selector(self.previous_buttons)
    }

    fn axis_pressed_left(&self) -> bool {
        self.left_stick_x < -STICK_DEADZONE && self.previous_left_stick_x >= -STICK_DEADZONE
    }

    fn axis_pressed_right(&self) -> bool {
        self.left_stick_x > STICK_DEADZONE && self.previous_left_stick_x <= STICK_DEADZONE
    }

    fn axis_pressed_up(&self) -> bool {
        self.left_stick_y > STICK_DEADZONE && self.previous_left_stick_y <= STICK_DEADZONE
    }

    fn axis_pressed_down(&self) -> bool {
        self.left_stick_y < -STICK_DEADZONE && self.previous_left_stick_y >= -STICK_DEADZONE
    }
}

pub struct GamepadInput {
    gilrs: Option<Gilrs>,
    active_id: Option<GamepadId>,
    previous_snapshot: GamepadSnapshot,
}

impl GamepadInput {
    pub fn new() -> Self {
        let gilrs = match Gilrs::new() {
            Ok(gilrs) => Some(gilrs),
            Err(error) => {
                eprintln!("Gamepad input disabled: {error}");
                None
            }
        };

        Self {
            gilrs,
            active_id: None,
            previous_snapshot: GamepadSnapshot::default(),
        }
    }

    pub fn update(&mut self) -> GamepadSnapshot {
        let previous_snapshot = self.previous_snapshot;
        let Some(gilrs) = &mut self.gilrs else {
            return previous_snapshot;
        };

        while let Some(Event { id, event, .. }) = gilrs.next_event() {
            match event {
                EventType::Connected => {
                    let gamepad = gilrs.gamepad(id);
                    println!("Gamepad connected: {}", gamepad.name());
                    self.active_id = Some(id);
                }
                EventType::Disconnected => {
                    if self.active_id == Some(id) {
                        println!("Gamepad disconnected");
                        self.active_id = None;
                    }
                }
                _ => {
                    if self.active_id.is_none() && gilrs.gamepad(id).is_connected() {
                        self.active_id = Some(id);
                    }
                }
            }
        }

        if self
            .active_id
            .is_some_and(|id| !gilrs.gamepad(id).is_connected())
        {
            self.active_id = None;
        }

        if self.active_id.is_none() {
            self.active_id = gilrs
                .gamepads()
                .find_map(|(id, gamepad)| gamepad.is_connected().then_some(id));
        }

        let snapshot = self
            .active_id
            .map(|id| {
                let gamepad = gilrs.gamepad(id);

                GamepadSnapshot {
                    left_stick_x: gamepad.value(Axis::LeftStickX),
                    left_stick_y: gamepad.value(Axis::LeftStickY),
                    right_stick_x: gamepad.value(Axis::RightStickX),
                    previous_left_stick_x: previous_snapshot.left_stick_x,
                    previous_left_stick_y: previous_snapshot.left_stick_y,
                    previous_buttons: previous_snapshot.buttons,
                    buttons: GamepadButtons {
                        south: gamepad.is_pressed(Button::South),
                        east: gamepad.is_pressed(Button::East),
                        north: gamepad.is_pressed(Button::North),
                        west: gamepad.is_pressed(Button::West),
                        select: gamepad.is_pressed(Button::Select),
                        start: gamepad.is_pressed(Button::Start),
                        dpad_left: gamepad.is_pressed(Button::DPadLeft),
                        dpad_right: gamepad.is_pressed(Button::DPadRight),
                        dpad_up: gamepad.is_pressed(Button::DPadUp),
                        dpad_down: gamepad.is_pressed(Button::DPadDown),
                        left_trigger: gamepad.is_pressed(Button::LeftTrigger)
                            || gamepad.value(Axis::LeftZ) > TRIGGER_DEADZONE,
                        right_trigger: gamepad.is_pressed(Button::RightTrigger)
                            || gamepad.value(Axis::RightZ) > TRIGGER_DEADZONE,
                        right_thumb: gamepad.is_pressed(Button::RightThumb),
                    },
                }
            })
            .unwrap_or_else(|| GamepadSnapshot {
                previous_buttons: previous_snapshot.buttons,
                ..GamepadSnapshot::default()
            });

        self.previous_snapshot = snapshot;
        snapshot
    }
}

fn apply_deadzone(value: f32, deadzone: f32) -> f32 {
    if value.abs() < deadzone { 0.0 } else { value }
}

fn digital_axis(positive: bool, negative: bool) -> f32 {
    match (positive, negative) {
        (true, false) => 1.0,
        (false, true) => -1.0,
        _ => 0.0,
    }
}

fn strongest_axis(first: f32, second: f32) -> f32 {
    if first.abs() >= second.abs() {
        first
    } else {
        second
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deadzone_suppresses_small_axis_values() {
        assert_eq!(apply_deadzone(0.05, 0.18), 0.0);
        assert_eq!(apply_deadzone(-0.05, 0.18), 0.0);
    }

    #[test]
    fn deadzone_preserves_values_outside_threshold() {
        assert_eq!(apply_deadzone(0.5, 0.18), 0.5);
        assert_eq!(apply_deadzone(-0.5, 0.18), -0.5);
    }

    #[test]
    fn digital_axis_prefers_single_active_direction() {
        assert_eq!(digital_axis(true, false), 1.0);
        assert_eq!(digital_axis(false, true), -1.0);
        assert_eq!(digital_axis(true, true), 0.0);
        assert_eq!(digital_axis(false, false), 0.0);
    }

    #[test]
    fn movement_axis_uses_left_stick_up_for_forward_motion() {
        let snapshot = GamepadSnapshot {
            left_stick_y: 0.8,
            ..GamepadSnapshot::default()
        };

        assert_eq!(snapshot.movement_axis(), 0.8);
    }

    #[test]
    fn movement_axis_accepts_dpad_buttons() {
        let snapshot = GamepadSnapshot {
            buttons: GamepadButtons {
                dpad_down: true,
                ..GamepadButtons::default()
            },
            ..GamepadSnapshot::default()
        };

        assert_eq!(snapshot.movement_axis(), -1.0);
    }

    #[test]
    fn button_pressed_reports_only_rising_edge() {
        let snapshot = GamepadSnapshot {
            previous_buttons: GamepadButtons::default(),
            buttons: GamepadButtons {
                south: true,
                ..GamepadButtons::default()
            },
            ..GamepadSnapshot::default()
        };

        assert!(snapshot.confirm_pressed());

        let held_snapshot = GamepadSnapshot {
            previous_buttons: snapshot.buttons,
            buttons: snapshot.buttons,
            ..GamepadSnapshot::default()
        };

        assert!(!held_snapshot.confirm_pressed());
    }
}
