use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::{Maze, cell_at_world_position};
use crate::player::Player;

const RAY_COLOR: u32 = 0xff0000;
const STEP_SIZE: f32 = 1.0;

#[derive(Debug, Clone, Copy)]
pub struct Intersect {
    pub distance: f32,
    pub impact: char,
}

pub fn cast_ray(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    angle: f32,
    block_size: usize,
    offset_x: usize,
    offset_y: usize,
    draw_line: bool,
) -> Intersect {
    let ray_cos = angle.cos();
    let ray_sin = angle.sin();
    let max_distance = max_ray_distance(maze, block_size);

    let mut distance = 0.0;
    let mut ray_x = player.pos.x;
    let mut ray_y = player.pos.y;

    while distance <= max_distance {
        distance += STEP_SIZE;
        ray_x = player.pos.x + distance * ray_cos;
        ray_y = player.pos.y + distance * ray_sin;

        match cell_at_world_position(maze, ray_x, ray_y, block_size) {
            Some(' ' | 'p' | 'g') => {}
            Some(impact) => {
                draw_ray_line(
                    framebuffer,
                    player,
                    ray_x,
                    ray_y,
                    offset_x,
                    offset_y,
                    draw_line,
                );

                return Intersect { distance, impact };
            }
            None => {
                draw_ray_line(
                    framebuffer,
                    player,
                    ray_x,
                    ray_y,
                    offset_x,
                    offset_y,
                    draw_line,
                );

                return Intersect {
                    distance,
                    impact: '#',
                };
            }
        }
    }

    draw_ray_line(
        framebuffer,
        player,
        ray_x,
        ray_y,
        offset_x,
        offset_y,
        draw_line,
    );

    Intersect {
        distance,
        impact: '#',
    }
}

fn max_ray_distance(maze: &Maze, block_size: usize) -> f32 {
    let maze_width = maze.iter().map(|row| row.len()).max().unwrap_or(0) * block_size;
    let maze_height = maze.len() * block_size;

    ((maze_width * maze_width + maze_height * maze_height) as f32).sqrt()
}

fn draw_ray_line(
    framebuffer: &mut Framebuffer,
    player: &Player,
    ray_x: f32,
    ray_y: f32,
    offset_x: usize,
    offset_y: usize,
    draw_line: bool,
) {
    if !draw_line {
        return;
    }

    let start_x = (player.pos.x + offset_x as f32).round() as isize;
    let start_y = (player.pos.y + offset_y as f32).round() as isize;
    let end_x = (ray_x + offset_x as f32).round() as isize;
    let end_y = (ray_y + offset_y as f32).round() as isize;

    framebuffer.set_current_color(RAY_COLOR);
    line(framebuffer, start_x, start_y, end_x, end_y);
}
