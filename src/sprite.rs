use crate::player::{Player, Vec2};

pub const FOOD_POWER_DURATION: f32 = 12.0;
pub const SPRITE_PICKUP_RADIUS: f32 = 18.0;
pub const SPRITE_EAT_RADIUS: f32 = 20.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteKind {
    Food,
    Ghost1,
}

#[derive(Clone, Debug)]
pub struct WorldSprite {
    pub kind: SpriteKind,
    pub pos: Vec2,
    pub active: bool,
}

impl WorldSprite {
    pub fn new(kind: SpriteKind, column: usize, row: usize, block_size: usize) -> Self {
        let block_size = block_size as f32;

        Self {
            kind,
            pos: Vec2 {
                x: column as f32 * block_size + block_size / 2.0,
                y: row as f32 * block_size + block_size / 2.0,
            },
            active: true,
        }
    }

    pub fn distance_to_player(&self, player: &Player) -> f32 {
        distance_between(self.pos, player.pos)
    }
}

#[derive(Debug, Clone)]
pub struct SpriteState {
    pub sprites: Vec<WorldSprite>,
    pub food_power_timer: f32,
}

impl SpriteState {
    pub fn for_level(level_index: usize, block_size: usize) -> Self {
        Self {
            sprites: level_sprites(level_index, block_size),
            food_power_timer: 0.0,
        }
    }

    pub fn reset_for_level(&mut self, level_index: usize, block_size: usize) {
        *self = Self::for_level(level_index, block_size);
    }

    pub fn has_food_power(&self) -> bool {
        self.food_power_timer > 0.0
    }

    pub fn update(&mut self, player: &Player, delta_time: f32) {
        self.food_power_timer = (self.food_power_timer - delta_time).max(0.0);

        for sprite in &mut self.sprites {
            if !sprite.active || sprite.kind != SpriteKind::Food {
                continue;
            }

            if sprite.distance_to_player(player) <= SPRITE_PICKUP_RADIUS {
                sprite.active = false;
                self.food_power_timer = FOOD_POWER_DURATION;
            }
        }

        if !self.has_food_power() {
            return;
        }

        for sprite in &mut self.sprites {
            if !sprite.active || sprite.kind != SpriteKind::Ghost1 {
                continue;
            }

            if sprite.distance_to_player(player) <= SPRITE_EAT_RADIUS {
                sprite.active = false;
            }
        }
    }
}

pub fn level_sprites(level_index: usize, block_size: usize) -> Vec<WorldSprite> {
    match level_index {
        0 => vec![
            WorldSprite::new(SpriteKind::Food, 3, 1, block_size),
            WorldSprite::new(SpriteKind::Ghost1, 7, 5, block_size),
        ],
        _ => Vec::new(),
    }
}

pub fn distance_between(first: Vec2, second: Vec2) -> f32 {
    let dx = first.x - second.x;
    let dy = first.y - second.y;

    (dx * dx + dy * dy).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_one_has_food_and_ghost() {
        let sprites = level_sprites(0, 40);

        assert_eq!(sprites.len(), 2);
        assert!(sprites.iter().any(|sprite| sprite.kind == SpriteKind::Food));
        assert!(
            sprites
                .iter()
                .any(|sprite| sprite.kind == SpriteKind::Ghost1)
        );
    }

    #[test]
    fn later_levels_start_without_sprites() {
        assert!(level_sprites(1, 40).is_empty());
        assert!(level_sprites(2, 40).is_empty());
    }

    #[test]
    fn sprite_positions_use_cell_centers() {
        let sprite = WorldSprite::new(SpriteKind::Food, 3, 1, 40);

        assert_eq!(sprite.pos.x, 140.0);
        assert_eq!(sprite.pos.y, 60.0);
    }

    #[test]
    fn sprite_state_resets_level_sprites_and_power() {
        let mut state = SpriteState::for_level(0, 40);
        state.food_power_timer = 4.0;
        state.sprites[0].active = false;

        state.reset_for_level(0, 40);

        assert_eq!(state.sprites.len(), 2);
        assert!(state.sprites.iter().all(|sprite| sprite.active));
        assert_eq!(state.food_power_timer, 0.0);
    }

    #[test]
    fn player_collects_food_and_gains_power() {
        let player = Player::new(3, 1, 40);
        let mut state = SpriteState::for_level(0, 40);

        state.update(&player, 0.0);

        assert!(state.has_food_power());
        assert!(!state.sprites[0].active);
        assert_eq!(state.food_power_timer, FOOD_POWER_DURATION);
    }

    #[test]
    fn food_power_timer_counts_down() {
        let player = Player::new(0, 0, 40);
        let mut state = SpriteState::for_level(0, 40);
        state.food_power_timer = 4.0;

        state.update(&player, 1.5);

        assert_eq!(state.food_power_timer, 2.5);
    }

    #[test]
    fn powered_player_eats_ghost() {
        let player = Player::new(7, 5, 40);
        let mut state = SpriteState::for_level(0, 40);
        state.food_power_timer = 4.0;

        state.update(&player, 0.0);

        assert!(!state.sprites[1].active);
    }

    #[test]
    fn unpowered_player_does_not_eat_ghost() {
        let player = Player::new(7, 5, 40);
        let mut state = SpriteState::for_level(0, 40);

        state.update(&player, 0.0);

        assert!(state.sprites[1].active);
    }
}
