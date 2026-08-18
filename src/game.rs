use crate::maze::{Maze, cell_at_world_position};
use crate::player::Player;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    Welcome,
    Playing,
    Won,
}

pub fn player_reached_goal(maze: &Maze, player: &Player, block_size: usize) -> bool {
    matches!(
        cell_at_world_position(maze, player.pos.x, player.pos.y, block_size),
        Some('g')
    )
}

pub fn reset_player(player: &mut Player, spawn: (usize, usize), block_size: usize) {
    let fov = player.fov;

    *player = Player::new(spawn.0, spawn.1, block_size);
    player.fov = fov;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_maze() -> Maze {
        vec![
            "#####".chars().collect(),
            "#p g#".chars().collect(),
            "#####".chars().collect(),
        ]
    }

    #[test]
    fn player_reached_goal_is_false_outside_goal() {
        let maze = test_maze();
        let player = Player::new(1, 1, 10);

        assert!(!player_reached_goal(&maze, &player, 10));
    }

    #[test]
    fn player_reached_goal_is_true_inside_goal() {
        let maze = test_maze();
        let player = Player::new(3, 1, 10);

        assert!(player_reached_goal(&maze, &player, 10));
    }

    #[test]
    fn player_reached_goal_is_false_outside_map() {
        let maze = test_maze();
        let mut player = Player::new(3, 1, 10);
        player.pos.x = -1.0;

        assert!(!player_reached_goal(&maze, &player, 10));
    }

    #[test]
    fn reset_player_restores_spawn_and_motion_state() {
        let mut player = Player::new(3, 1, 10);
        let initial_angle = Player::new(1, 1, 10).a;
        player.a = 2.5;
        player.fov = 0.9;
        player.height = 40.0;
        player.vertical_velocity = -120.0;
        player.is_grounded = false;

        reset_player(&mut player, (1, 1), 10);

        assert_eq!(player.pos.x, 15.0);
        assert_eq!(player.pos.y, 15.0);
        assert_eq!(player.a, initial_angle);
        assert_eq!(player.fov, 0.9);
        assert_eq!(player.height, 0.0);
        assert_eq!(player.vertical_velocity, 0.0);
        assert!(player.is_grounded);
    }
}
