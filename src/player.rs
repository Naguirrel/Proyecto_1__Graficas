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
        }
    }
}
