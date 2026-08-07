use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::Maze;
use crate::player::Player;

const WALL_COLOR: u32 = 0x3b82f6;
const WALL_PLUS_COLOR: u32 = 0xef4444;
const WALL_PERCENT_COLOR: u32 = 0x22c55e;
const WALL_AT_COLOR: u32 = 0xa855f7;
const PATH_COLOR: u32 = 0xd1d5db;
const GOAL_COLOR: u32 = 0xfacc15;
const PLAYER_COLOR: u32 = 0x00e5ff;
const PLAYER_DIRECTION_COLOR: u32 = 0xfff7ed;
const CEILING_COLOR: u32 = 0x111827;
const FLOOR_COLOR: u32 = 0x374151;
const UNKNOWN_COLOR: u32 = 0xff00ff;
const PLAYER_SIZE: isize = 4;
const DIRECTION_LENGTH: f32 = 30.0;
const JUMP_VISUAL_SCALE: f32 = 1.0;

pub fn render_maze(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize) {
    if maze.is_empty() || block_size == 0 {
        return;
    }

    let (offset_x, offset_y) = maze_offset(framebuffer, maze, block_size);

    for (row_index, row) in maze.iter().enumerate() {
        for (column_index, cell) in row.iter().enumerate() {
            let x0 = offset_x + column_index * block_size;
            let y0 = offset_y + row_index * block_size;

            draw_cell(framebuffer, x0, y0, block_size, *cell);
        }
    }
}

pub fn maze_offset(framebuffer: &Framebuffer, maze: &Maze, block_size: usize) -> (usize, usize) {
    let maze_width = maze.iter().map(|row| row.len()).max().unwrap_or(0) * block_size;
    let maze_height = maze.len() * block_size;

    (
        framebuffer.width.saturating_sub(maze_width) / 2,
        framebuffer.height.saturating_sub(maze_height) / 2,
    )
}

pub fn render_player(
    framebuffer: &mut Framebuffer,
    player: &Player,
    offset_x: usize,
    offset_y: usize,
) {
    let screen_x = (player.pos.x + offset_x as f32).round() as isize;
    let screen_y = (player.pos.y + offset_y as f32).round() as isize;

    framebuffer.set_current_color(PLAYER_COLOR);

    for y in -PLAYER_SIZE..=PLAYER_SIZE {
        for x in -PLAYER_SIZE..=PLAYER_SIZE {
            framebuffer.point(screen_x + x, screen_y + y);
        }
    }

    let end_x = player.pos.x + player.a.cos() * DIRECTION_LENGTH;
    let end_y = player.pos.y + player.a.sin() * DIRECTION_LENGTH;
    let screen_end_x = (end_x + offset_x as f32).round() as isize;
    let screen_end_y = (end_y + offset_y as f32).round() as isize;

    framebuffer.set_current_color(PLAYER_DIRECTION_COLOR);
    line(framebuffer, screen_x, screen_y, screen_end_x, screen_end_y);
}

pub fn render_3d(framebuffer: &mut Framebuffer, maze: &Maze, player: &Player, block_size: usize) {
    if framebuffer.width == 0 || framebuffer.height == 0 {
        return;
    }

    let projection_plane = distance_to_projection_plane(framebuffer.width, player.fov);
    let screen_center = framebuffer.height as f32 / 2.0 + player.height * JUMP_VISUAL_SCALE;
    let last_column = framebuffer.width.saturating_sub(1);

    render_3d_background(framebuffer);

    for x in 0..framebuffer.width {
        let fraction = if framebuffer.width == 1 {
            0.5
        } else {
            x as f32 / last_column as f32
        };

        let ray_angle = player.a - player.fov / 2.0 + fraction * player.fov;
        let intersection = cast_ray(
            framebuffer,
            maze,
            player,
            ray_angle,
            block_size,
            0,
            0,
            false,
        );
        let corrected_distance = correct_fish_eye(intersection.distance, ray_angle, player.a);
        let stake_height =
            projected_stake_height(block_size as f32, corrected_distance, projection_plane);
        let stake_top = (screen_center - stake_height / 2.0).max(0.0).round() as isize;
        let stake_bottom = (screen_center + stake_height / 2.0)
            .min((framebuffer.height - 1) as f32)
            .round() as isize;

        framebuffer.set_current_color(wall_color(intersection.impact));
        line(framebuffer, x as isize, stake_top, x as isize, stake_bottom);
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, x0: usize, y0: usize, block_size: usize, cell: char) {
    let color = match cell {
        '#' => WALL_COLOR,
        '+' => WALL_PLUS_COLOR,
        '%' => WALL_PERCENT_COLOR,
        '@' => WALL_AT_COLOR,
        ' ' | 'p' => PATH_COLOR,
        'g' => GOAL_COLOR,
        _ => UNKNOWN_COLOR,
    };

    framebuffer.set_current_color(color);

    // Dejamos una linea de fondo entre celdas para inspeccionar mejor el mapa.
    let visible_size = block_size.saturating_sub(1);

    for y in 0..visible_size {
        for x in 0..visible_size {
            framebuffer.point((x0 + x) as isize, (y0 + y) as isize);
        }
    }
}

fn render_3d_background(framebuffer: &mut Framebuffer) {
    let half_height = framebuffer.height / 2;

    framebuffer.set_current_color(CEILING_COLOR);
    for y in 0..half_height {
        for x in 0..framebuffer.width {
            framebuffer.point(x as isize, y as isize);
        }
    }

    framebuffer.set_current_color(FLOOR_COLOR);
    for y in half_height..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.point(x as isize, y as isize);
        }
    }
}

fn distance_to_projection_plane(screen_width: usize, fov: f32) -> f32 {
    let half_fov_tan = (fov / 2.0).tan().max(0.0001);

    (screen_width as f32 / 2.0) / half_fov_tan
}

fn correct_fish_eye(distance: f32, ray_angle: f32, player_angle: f32) -> f32 {
    distance * (ray_angle - player_angle).cos()
}

fn projected_stake_height(wall_height: f32, distance_to_wall: f32, projection_plane: f32) -> f32 {
    let distance_to_wall = distance_to_wall.max(0.0001);

    (wall_height / distance_to_wall) * projection_plane
}

fn wall_color(impact: char) -> u32 {
    match impact {
        '#' => WALL_COLOR,
        '+' => WALL_PLUS_COLOR,
        '%' => WALL_PERCENT_COLOR,
        '@' => WALL_AT_COLOR,
        _ => UNKNOWN_COLOR,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn projection_plane_is_positive_for_valid_fov() {
        let distance = distance_to_projection_plane(800, std::f32::consts::PI / 3.0);

        assert!(distance > 0.0);
    }

    #[test]
    fn closer_wall_projects_taller_stake() {
        let projection_plane = distance_to_projection_plane(800, std::f32::consts::PI / 3.0);
        let near_stake = projected_stake_height(40.0, 40.0, projection_plane);
        let far_stake = projected_stake_height(40.0, 160.0, projection_plane);

        assert!(near_stake > far_stake);
    }

    #[test]
    fn fish_eye_correction_preserves_center_distance_and_reduces_side_distance() {
        let center = correct_fish_eye(100.0, 1.0, 1.0);
        let side = correct_fish_eye(100.0, 1.4, 1.0);

        assert_eq!(center, 100.0);
        assert!(side < 100.0);
    }

    #[test]
    fn wall_characters_have_distinct_colors() {
        assert_eq!(wall_color('#'), WALL_COLOR);
        assert_eq!(wall_color('+'), WALL_PLUS_COLOR);
        assert_eq!(wall_color('%'), WALL_PERCENT_COLOR);
        assert_eq!(wall_color('@'), WALL_AT_COLOR);
        assert_ne!(wall_color('#'), wall_color('+'));
        assert_ne!(wall_color('+'), wall_color('%'));
        assert_ne!(wall_color('%'), wall_color('@'));
    }
}
