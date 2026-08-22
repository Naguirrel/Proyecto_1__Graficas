use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::Maze;
use crate::player::Player;

const RAY_COLOR: u32 = 0xff0000;
const STEP_SIZE: f32 = 1.0;
const RAY_AXIS_EPSILON: f32 = 0.000001;
pub const NUM_RAYS_2D: usize = 60;

#[derive(Debug, Clone, Copy)]
#[allow(dead_code)]
pub struct Intersect {
    pub distance: f32,
    pub impact: char,
    pub hit_x: f32,
    pub hit_y: f32,
    pub side: WallSide,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WallSide {
    Vertical,
    Horizontal,
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
    if block_size == 0 || !player.pos.x.is_finite() || !player.pos.y.is_finite() {
        return Intersect {
            distance: 0.0,
            impact: '#',
            hit_x: player.pos.x,
            hit_y: player.pos.y,
            side: WallSide::Vertical,
        };
    }

    let ray_cos = angle.cos();
    let ray_sin = angle.sin();
    let max_distance = max_ray_distance(maze, block_size);
    let block_size_f = block_size as f32;
    let mut map_x = (player.pos.x / block_size_f).floor() as isize;
    let mut map_y = (player.pos.y / block_size_f).floor() as isize;

    let step_x = if ray_cos > RAY_AXIS_EPSILON {
        1
    } else if ray_cos < -RAY_AXIS_EPSILON {
        -1
    } else {
        0
    };
    let step_y = if ray_sin > RAY_AXIS_EPSILON {
        1
    } else if ray_sin < -RAY_AXIS_EPSILON {
        -1
    } else {
        0
    };

    let mut side_distance_x =
        first_side_distance(player.pos.x, ray_cos, map_x, block_size_f, step_x);
    let mut side_distance_y =
        first_side_distance(player.pos.y, ray_sin, map_y, block_size_f, step_y);
    let delta_distance_x = axis_delta_distance(ray_cos, block_size_f, step_x);
    let delta_distance_y = axis_delta_distance(ray_sin, block_size_f, step_y);

    while side_distance_x.min(side_distance_y) <= max_distance {
        let (distance, side) = if side_distance_x <= side_distance_y {
            let distance = side_distance_x;
            side_distance_x += delta_distance_x;
            map_x += step_x;

            (distance, WallSide::Vertical)
        } else {
            let distance = side_distance_y;
            side_distance_y += delta_distance_y;
            map_y += step_y;

            (distance, WallSide::Horizontal)
        };

        let ray_x = player.pos.x + distance * ray_cos;
        let ray_y = player.pos.y + distance * ray_sin;

        match cell_at_grid_position(maze, map_x, map_y) {
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

                return Intersect {
                    distance,
                    impact,
                    hit_x: ray_x,
                    hit_y: ray_y,
                    side,
                };
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
                    hit_x: ray_x,
                    hit_y: ray_y,
                    side,
                };
            }
        }
    }

    let distance = max_distance;
    let ray_x = player.pos.x + distance * ray_cos;
    let ray_y = player.pos.y + distance * ray_sin;

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
        hit_x: ray_x,
        hit_y: ray_y,
        side: WallSide::Vertical,
    }
}

pub fn cast_fov_2d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    offset_x: usize,
    offset_y: usize,
) {
    for ray_index in 0..NUM_RAYS_2D {
        let ray_angle = fov_ray_angle(player.a, player.fov, ray_index, NUM_RAYS_2D);

        cast_ray(
            framebuffer,
            maze,
            player,
            ray_angle,
            block_size,
            offset_x,
            offset_y,
            true,
        );
    }
}

fn fov_ray_angle(player_angle: f32, fov: f32, ray_index: usize, ray_count: usize) -> f32 {
    let fraction = if ray_count == 1 {
        0.5
    } else {
        ray_index as f32 / (ray_count - 1) as f32
    };

    player_angle - fov / 2.0 + fraction * fov
}

fn max_ray_distance(maze: &Maze, block_size: usize) -> f32 {
    let maze_width = maze.iter().map(|row| row.len()).max().unwrap_or(0) * block_size;
    let maze_height = maze.len() * block_size;

    ((maze_width * maze_width + maze_height * maze_height) as f32).sqrt()
}

fn first_side_distance(
    position: f32,
    ray_direction: f32,
    map_position: isize,
    block_size: f32,
    step: isize,
) -> f32 {
    match step {
        1 => (((map_position + 1) as f32 * block_size) - position) / ray_direction,
        -1 => (position - map_position as f32 * block_size) / -ray_direction,
        _ => f32::INFINITY,
    }
}

fn axis_delta_distance(ray_direction: f32, block_size: f32, step: isize) -> f32 {
    if step == 0 {
        f32::INFINITY
    } else {
        block_size / ray_direction.abs()
    }
}

fn cell_at_grid_position(maze: &Maze, column: isize, row: isize) -> Option<char> {
    if column < 0 || row < 0 {
        return None;
    }

    maze.get(row as usize)
        .and_then(|maze_row| maze_row.get(column as usize))
        .copied()
}

fn wall_side_at_hit(hit_x: f32, hit_y: f32, block_size: usize) -> WallSide {
    if block_size == 0 || !hit_x.is_finite() || !hit_y.is_finite() {
        return WallSide::Vertical;
    }

    let cell_size = block_size as f32;
    let local_x = hit_x.rem_euclid(cell_size);
    let local_y = hit_y.rem_euclid(cell_size);
    let vertical_distance = local_x.min(cell_size - local_x);
    let horizontal_distance = local_y.min(cell_size - local_y);

    // En esquinas o empates priorizamos Vertical para que la clasificacion sea determinista.
    if vertical_distance <= horizontal_distance {
        WallSide::Vertical
    } else {
        WallSide::Horizontal
    }
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

    fn assert_hit_position_close(intersection: Intersect, expected_x: f32, expected_y: f32) {
        assert!(
            (intersection.hit_x - expected_x).abs() <= STEP_SIZE,
            "expected hit_x near {expected_x}, got {}",
            intersection.hit_x
        );
        assert!(
            (intersection.hit_y - expected_y).abs() <= STEP_SIZE,
            "expected hit_y near {expected_y}, got {}",
            intersection.hit_y
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
    fn cast_ray_reports_right_hit_position_in_world_space() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let intersection = cast_ray(&mut framebuffer, &maze, &player, 0.0, 10, 9, 11, false);

        assert_eq!(intersection.impact, '#');
        assert_hit_position_close(intersection, 40.0, 25.0);
        assert!(intersection.distance.is_finite());
        assert!(intersection.hit_x.is_finite());
        assert!(intersection.hit_y.is_finite());
    }

    #[test]
    fn cast_ray_reports_left_hit_position_in_world_space() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let intersection = cast_ray(&mut framebuffer, &maze, &player, PI, 10, 0, 0, false);

        assert_eq!(intersection.impact, '#');
        assert_hit_position_close(intersection, 9.0, 25.0);
        assert!(intersection.distance.is_finite());
        assert!(intersection.hit_x.is_finite());
        assert!(intersection.hit_y.is_finite());
    }

    #[test]
    fn cast_ray_reports_down_hit_position_in_world_space() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let intersection = cast_ray(&mut framebuffer, &maze, &player, PI / 2.0, 10, 0, 0, false);

        assert_eq!(intersection.impact, '#');
        assert_hit_position_close(intersection, 25.0, 40.0);
        assert!(intersection.distance.is_finite());
        assert!(intersection.hit_x.is_finite());
        assert!(intersection.hit_y.is_finite());
    }

    #[test]
    fn cast_ray_reports_up_hit_position_in_world_space() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let intersection = cast_ray(
            &mut framebuffer,
            &maze,
            &player,
            3.0 * PI / 2.0,
            10,
            0,
            0,
            false,
        );

        assert_eq!(intersection.impact, '#');
        assert_hit_position_close(intersection, 25.0, 9.0);
        assert!(intersection.distance.is_finite());
        assert!(intersection.hit_x.is_finite());
        assert!(intersection.hit_y.is_finite());
    }

    #[test]
    fn cast_ray_reports_vertical_side_for_left_and_right_hits() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let right = cast_ray(&mut framebuffer, &maze, &player, 0.0, 10, 0, 0, false);
        let left = cast_ray(&mut framebuffer, &maze, &player, PI, 10, 0, 0, false);

        assert_eq!(right.side, WallSide::Vertical);
        assert_eq!(left.side, WallSide::Vertical);
    }

    #[test]
    fn cast_ray_reports_horizontal_side_for_up_and_down_hits() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let down = cast_ray(&mut framebuffer, &maze, &player, PI / 2.0, 10, 0, 0, false);
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

        assert_eq!(down.side, WallSide::Horizontal);
        assert_eq!(up.side, WallSide::Horizontal);
    }

    #[test]
    fn wall_side_tie_prefers_vertical() {
        assert_eq!(wall_side_at_hit(5.0, 5.0, 10), WallSide::Vertical);
    }

    #[test]
    fn cast_ray_with_zero_block_size_returns_finite_hit_info() {
        let maze = test_maze();
        let player = test_player();
        let mut framebuffer = Framebuffer::new(80, 80);

        let intersection = cast_ray(&mut framebuffer, &maze, &player, 0.0, 0, 0, 0, false);

        assert_eq!(intersection.impact, '#');
        assert_eq!(intersection.side, WallSide::Vertical);
        assert!(intersection.distance.is_finite());
        assert!(intersection.hit_x.is_finite());
        assert!(intersection.hit_y.is_finite());
    }

    #[test]
    fn cast_ray_outside_maze_returns_finite_fallback_hit_info() {
        let maze = vec!["p ".chars().collect()];
        let player = Player::new(0, 0, 10);
        let mut framebuffer = Framebuffer::new(80, 80);

        let intersection = cast_ray(&mut framebuffer, &maze, &player, 0.0, 10, 0, 0, false);

        assert_eq!(intersection.impact, '#');
        assert!(matches!(
            intersection.side,
            WallSide::Vertical | WallSide::Horizontal
        ));
        assert!(intersection.distance.is_finite());
        assert!(intersection.hit_x.is_finite());
        assert!(intersection.hit_y.is_finite());
    }

    #[test]
    fn cast_fov_uses_expected_number_of_rays() {
        assert_eq!(NUM_RAYS_2D, 60);
    }

    #[test]
    fn fov_2d_covers_full_fov() {
        let player = test_player();
        let first_angle = fov_ray_angle(player.a, player.fov, 0, NUM_RAYS_2D);
        let last_angle = fov_ray_angle(player.a, player.fov, NUM_RAYS_2D - 1, NUM_RAYS_2D);

        assert_eq!(first_angle, player.a - player.fov / 2.0);
        assert_eq!(last_angle, player.a + player.fov / 2.0);
    }
}
