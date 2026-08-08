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

#[allow(clippy::too_many_arguments)]
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

pub fn cast_fov_2d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    offset_x: usize,
    offset_y: usize,
) -> Vec<Intersect> {
    let ray_count = framebuffer.width;
    let mut intersects = Vec::with_capacity(ray_count);

    if ray_count == 0 {
        return intersects;
    }

    for ray_index in 0..ray_count {
        let fraction = if ray_count == 1 {
            0.5
        } else {
            ray_index as f32 / (ray_count - 1) as f32
        };

        let ray_angle = player.a - player.fov / 2.0 + fraction * player.fov;
        let intersect = cast_ray(
            framebuffer,
            maze,
            player,
            ray_angle,
            block_size,
            offset_x,
            offset_y,
            true,
        );

        intersects.push(intersect);
    }

    intersects
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

#[cfg(test)]
mod tests {
    use std::f32::consts::PI;

    use super::*;

    fn test_maze() -> Maze {
        vec![
            "#####".chars().collect(),
            "#   #".chars().collect(),
            "# p #".chars().collect(),
            "#   #".chars().collect(),
            "#####".chars().collect(),
        ]
    }

    fn test_player() -> Player {
        Player::new(2, 2, 10)
    }

    fn assert_distance_close(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() <= STEP_SIZE,
            "expected distance near {expected}, got {actual}"
        );
    }

    #[test]
    fn cast_ray_hits_first_wall_in_cardinal_directions() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let right = cast_ray(&mut framebuffer, &maze, &player, 0.0, 10, 0, 0, false);
        assert_eq!(right.impact, '#');
        assert_distance_close(right.distance, 15.0);

        let down = cast_ray(&mut framebuffer, &maze, &player, PI / 2.0, 10, 0, 0, false);
        assert_eq!(down.impact, '#');
        assert_distance_close(down.distance, 15.0);

        let left = cast_ray(&mut framebuffer, &maze, &player, PI, 10, 0, 0, false);
        assert_eq!(left.impact, '#');
        assert_distance_close(left.distance, 16.0);

        let up = cast_ray(
            &mut framebuffer,
            &maze,
            &player,
            3.0 * PI / 2.0,
            10,
            0,
            0,
            false,
        );
        assert_eq!(up.impact, '#');
        assert_distance_close(up.distance, 16.0);
    }

    #[test]
    fn cast_ray_with_draw_line_false_does_not_modify_framebuffer() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        cast_ray(&mut framebuffer, &maze, &player, 0.0, 10, 0, 0, false);

        assert!(framebuffer.buffer.iter().all(|pixel| *pixel == 0));
    }

    #[test]
    fn cast_fov_uses_expected_number_of_rays() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let rays = cast_fov_2d(&mut framebuffer, &maze, &player, 10, 0, 0);

        assert_eq!(rays.len(), framebuffer.width);
        assert!(rays.iter().all(|ray| ray.impact == '#'));
    }
}
