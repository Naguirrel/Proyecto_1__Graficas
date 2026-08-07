use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}

pub struct Player {
    pub pos: Vec2,
    pub a: f32,
    pub fov: f32,
    pub height: f32,
    pub vertical_velocity: f32,
    pub is_grounded: bool,
}

impl Player {
    pub fn new(column: usize, row: usize, block_size: usize) -> Self {
        let block_size = block_size as f32;

        Self {
            pos: Vec2 {
                x: column as f32 * block_size + block_size / 2.0,
                y: row as f32 * block_size + block_size / 2.0,
            },
            a: PI / 3.0,
            fov: PI / 3.0,
            height: 0.0,
            vertical_velocity: 0.0,
            is_grounded: true,
        }
    }

    pub fn start_jump(&mut self, jump_speed: f32) {
        if self.is_grounded {
            self.vertical_velocity = jump_speed;
            self.is_grounded = false;
        }
    }

    pub fn update_vertical_motion(&mut self, delta_time: f32, gravity: f32) {
        self.height += self.vertical_velocity * delta_time;
        self.vertical_velocity -= gravity * delta_time;

        if self.height <= 0.0 {
            self.height = 0.0;
            self.vertical_velocity = 0.0;
            self.is_grounded = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grounded_player_can_start_jump() {
        let mut player = Player::new(1, 1, 40);

        player.start_jump(220.0);

        assert_eq!(player.vertical_velocity, 220.0);
        assert!(!player.is_grounded);
    }

    #[test]
    fn player_cannot_double_jump_while_airborne() {
        let mut player = Player::new(1, 1, 40);

        player.start_jump(220.0);
        player.start_jump(999.0);

        assert_eq!(player.vertical_velocity, 220.0);
    }

    #[test]
    fn vertical_motion_returns_player_to_ground() {
        let mut player = Player::new(1, 1, 40);
        player.start_jump(220.0);

        for _ in 0..120 {
            player.update_vertical_motion(1.0 / 60.0, 600.0);
        }

        assert_eq!(player.height, 0.0);
        assert_eq!(player.vertical_velocity, 0.0);
        assert!(player.is_grounded);
    }
}
