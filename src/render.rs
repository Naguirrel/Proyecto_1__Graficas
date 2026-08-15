use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::line::{line, line_with_shader};
use crate::maze::Maze;
use crate::player::Player;
use crate::texture::{TextureManager, texture_x_from_hit, texture_y_for_stake};

const WALL_COLOR: u32 = 0x3b82f6;
const WALL_PLUS_COLOR: u32 = 0xef4444;
const WALL_PERCENT_COLOR: u32 = 0x22c55e;
const WALL_AT_COLOR: u32 = 0xa855f7;
const WALL_AMPERSAND_COLOR: u32 = 0xf97316;
const GOAL_WALL_COLOR: u32 = 0xfacc15;
const PATH_COLOR: u32 = 0xd1d5db;
const GOAL_COLOR: u32 = 0xfacc15;
const PLAYER_COLOR: u32 = 0x00e5ff;
const PLAYER_DIRECTION_COLOR: u32 = 0xfff7ed;
const CEILING_COLOR: u32 = 0x111827;
const FLOOR_COLOR: u32 = 0x374151;
const UNKNOWN_COLOR: u32 = 0xff00ff;
const VICTORY_BACKGROUND_COLOR: u32 = 0x052e2b;
const VICTORY_ACCENT_COLOR: u32 = 0xfacc15;
const VICTORY_TEXT_COLOR: u32 = 0xfff7ed;
const VICTORY_SHADOW_COLOR: u32 = 0x0f172a;
const FPS_OVERLAY_BACKGROUND_COLOR: u32 = 0x111111;
const FPS_OVERLAY_TEXT_COLOR: u32 = 0xfff7ed;
const FPS_OVERLAY_X: isize = 10;
const FPS_OVERLAY_Y: isize = 10;
const FPS_OVERLAY_PADDING: isize = 6;
const FPS_OVERLAY_SCALE: isize = 3;
const MINIMAP_CELL_SIZE: usize = 6;
const MINIMAP_MARGIN: usize = 10;
const MINIMAP_PADDING: usize = 4;
const MINIMAP_BACKGROUND_COLOR: u32 = 0x111111;
const MINIMAP_BORDER_COLOR: u32 = 0xfff7ed;
const MINIMAP_WALL_COLOR: u32 = 0xd1d5db;
const MINIMAP_PATH_COLOR: u32 = 0x1f2937;
const MINIMAP_GOAL_COLOR: u32 = 0xfacc15;
const MINIMAP_PLAYER_COLOR: u32 = PLAYER_COLOR;
const MINIMAP_DIRECTION_COLOR: u32 = PLAYER_DIRECTION_COLOR;
const MINIMAP_PLAYER_SIZE: isize = 2;
const MINIMAP_DIRECTION_LENGTH: f32 = 10.0;
const PLAYER_SIZE: isize = 4;
const DIRECTION_LENGTH: f32 = 30.0;
const JUMP_VISUAL_SCALE: f32 = 1.0;
const GLYPH_WIDTH: isize = 5;
const GLYPH_HEIGHT: isize = 7;

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

pub fn render_3d(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    textures: &TextureManager,
) {
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
        let unclipped_stake_top = screen_center - stake_height / 2.0;
        let unclipped_stake_bottom = screen_center + stake_height / 2.0;
        let clipped_top = unclipped_stake_top.max(0.0).round() as isize;
        let clipped_bottom = unclipped_stake_bottom
            .min((framebuffer.height - 1) as f32)
            .round() as isize;
        let texture = textures.get(intersection.impact);
        let tx = texture_x_from_hit(
            intersection.hit_x,
            intersection.hit_y,
            intersection.side,
            block_size,
            texture.width,
        );

        line_with_shader(
            framebuffer,
            x as isize,
            clipped_top,
            x as isize,
            clipped_bottom,
            |_, pixel_y| {
                let ty = texture_y_for_stake(
                    pixel_y,
                    unclipped_stake_top,
                    unclipped_stake_bottom,
                    texture.height,
                );

                texture.get_pixel(tx, ty)
            },
        );
    }
}

pub fn render_victory_screen(framebuffer: &mut Framebuffer) {
    fill_screen(framebuffer, VICTORY_BACKGROUND_COLOR);

    let margin = 24;
    framebuffer.set_current_color(VICTORY_ACCENT_COLOR);
    line(
        framebuffer,
        margin,
        margin,
        framebuffer.width as isize - margin,
        margin,
    );
    line(
        framebuffer,
        margin,
        framebuffer.height as isize - margin,
        framebuffer.width as isize - margin,
        framebuffer.height as isize - margin,
    );
    line(
        framebuffer,
        margin,
        margin,
        margin,
        framebuffer.height as isize - margin,
    );
    line(
        framebuffer,
        framebuffer.width as isize - margin,
        margin,
        framebuffer.width as isize - margin,
        framebuffer.height as isize - margin,
    );

    draw_text_centered(framebuffer, "GANASTE", 150, 12, VICTORY_TEXT_COLOR);
    draw_text_centered(framebuffer, "R REINICIAR", 330, 5, VICTORY_ACCENT_COLOR);
    draw_text_centered(framebuffer, "ESC SALIR", 390, 5, VICTORY_TEXT_COLOR);
}

pub fn render_fps_overlay(framebuffer: &mut Framebuffer, fps: u32) {
    let text = format!("FPS: {fps}");
    let text_width = text_width(&text, FPS_OVERLAY_SCALE);
    let text_height = GLYPH_HEIGHT * FPS_OVERLAY_SCALE;
    let background_x = FPS_OVERLAY_X - FPS_OVERLAY_PADDING;
    let background_y = FPS_OVERLAY_Y - FPS_OVERLAY_PADDING;
    let background_width = text_width + FPS_OVERLAY_PADDING * 2;
    let background_height = text_height + FPS_OVERLAY_PADDING * 2;

    fill_rect(
        framebuffer,
        background_x,
        background_y,
        background_width,
        background_height,
        FPS_OVERLAY_BACKGROUND_COLOR,
    );
    draw_text(
        framebuffer,
        &text,
        FPS_OVERLAY_X,
        FPS_OVERLAY_Y,
        FPS_OVERLAY_SCALE,
        FPS_OVERLAY_TEXT_COLOR,
    );
}

pub fn render_minimap(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
) {
    if maze.is_empty() || block_size == 0 || framebuffer.width == 0 || framebuffer.height == 0 {
        return;
    }

    let (minimap_width, minimap_height) = minimap_dimensions(maze);

    if minimap_width == 0 || minimap_height == 0 {
        return;
    }

    let (content_x, content_y) = minimap_content_offset(framebuffer, minimap_width, minimap_height);
    let panel_x = content_x - MINIMAP_PADDING as isize;
    let panel_y = content_y - MINIMAP_PADDING as isize;
    let panel_width = (minimap_width + MINIMAP_PADDING * 2) as isize;
    let panel_height = (minimap_height + MINIMAP_PADDING * 2) as isize;

    fill_rect(
        framebuffer,
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        MINIMAP_BACKGROUND_COLOR,
    );
    draw_rect_outline(
        framebuffer,
        panel_x,
        panel_y,
        panel_width,
        panel_height,
        MINIMAP_BORDER_COLOR,
    );
    render_minimap_maze(framebuffer, maze, content_x, content_y);
    render_minimap_player(framebuffer, player, block_size, content_x, content_y);
}

fn draw_cell(framebuffer: &mut Framebuffer, x0: usize, y0: usize, block_size: usize, cell: char) {
    let color = match cell {
        '#' => WALL_COLOR,
        '+' => WALL_PLUS_COLOR,
        '%' => WALL_PERCENT_COLOR,
        '@' => WALL_AT_COLOR,
        '&' => WALL_AMPERSAND_COLOR,
        '!' => GOAL_WALL_COLOR,
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

fn render_minimap_maze(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    offset_x: isize,
    offset_y: isize,
) {
    for (row_index, row) in maze.iter().enumerate() {
        for (column_index, cell) in row.iter().enumerate() {
            let x = offset_x + (column_index * MINIMAP_CELL_SIZE) as isize;
            let y = offset_y + (row_index * MINIMAP_CELL_SIZE) as isize;

            fill_rect(
                framebuffer,
                x,
                y,
                MINIMAP_CELL_SIZE as isize,
                MINIMAP_CELL_SIZE as isize,
                minimap_cell_color(*cell),
            );
        }
    }
}

fn render_minimap_player(
    framebuffer: &mut Framebuffer,
    player: &Player,
    block_size: usize,
    offset_x: isize,
    offset_y: isize,
) {
    let Some((player_x, player_y)) =
        minimap_player_position(player, block_size, offset_x, offset_y)
    else {
        return;
    };

    framebuffer.set_current_color(MINIMAP_PLAYER_COLOR);

    for y in -MINIMAP_PLAYER_SIZE..=MINIMAP_PLAYER_SIZE {
        for x in -MINIMAP_PLAYER_SIZE..=MINIMAP_PLAYER_SIZE {
            framebuffer.point(player_x + x, player_y + y);
        }
    }

    let direction_end_x = player_x + (player.a.cos() * MINIMAP_DIRECTION_LENGTH).round() as isize;
    let direction_end_y = player_y + (player.a.sin() * MINIMAP_DIRECTION_LENGTH).round() as isize;

    framebuffer.set_current_color(MINIMAP_DIRECTION_COLOR);
    line(
        framebuffer,
        player_x,
        player_y,
        direction_end_x,
        direction_end_y,
    );
}

fn minimap_dimensions(maze: &Maze) -> (usize, usize) {
    let width = maze.iter().map(|row| row.len()).max().unwrap_or(0) * MINIMAP_CELL_SIZE;
    let height = maze.len() * MINIMAP_CELL_SIZE;

    (width, height)
}

fn minimap_content_offset(
    framebuffer: &Framebuffer,
    minimap_width: usize,
    minimap_height: usize,
) -> (isize, isize) {
    let panel_width = minimap_width + MINIMAP_PADDING * 2;
    let panel_height = minimap_height + MINIMAP_PADDING * 2;
    let panel_x = framebuffer
        .width
        .saturating_sub(panel_width + MINIMAP_MARGIN);
    let panel_y = framebuffer
        .height
        .saturating_sub(panel_height + MINIMAP_MARGIN)
        .min(MINIMAP_MARGIN);

    (
        (panel_x + MINIMAP_PADDING) as isize,
        (panel_y + MINIMAP_PADDING) as isize,
    )
}

fn minimap_player_position(
    player: &Player,
    block_size: usize,
    offset_x: isize,
    offset_y: isize,
) -> Option<(isize, isize)> {
    if block_size == 0 || !player.pos.x.is_finite() || !player.pos.y.is_finite() {
        return None;
    }

    let maze_x = player.pos.x / block_size as f32;
    let maze_y = player.pos.y / block_size as f32;

    Some((
        offset_x + (maze_x * MINIMAP_CELL_SIZE as f32).round() as isize,
        offset_y + (maze_y * MINIMAP_CELL_SIZE as f32).round() as isize,
    ))
}

fn minimap_cell_color(cell: char) -> u32 {
    match cell {
        '#' | '+' | '%' | '@' | '&' | '!' => MINIMAP_WALL_COLOR,
        'g' => MINIMAP_GOAL_COLOR,
        ' ' | 'p' => MINIMAP_PATH_COLOR,
        _ => UNKNOWN_COLOR,
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

#[allow(dead_code)]
fn wall_color(impact: char) -> u32 {
    match impact {
        '#' => WALL_COLOR,
        '+' => WALL_PLUS_COLOR,
        '%' => WALL_PERCENT_COLOR,
        '@' => WALL_AT_COLOR,
        '&' => WALL_AMPERSAND_COLOR,
        '!' => GOAL_WALL_COLOR,
        _ => UNKNOWN_COLOR,
    }
}

fn fill_screen(framebuffer: &mut Framebuffer, color: u32) {
    framebuffer.set_current_color(color);

    for y in 0..framebuffer.height {
        for x in 0..framebuffer.width {
            framebuffer.point(x as isize, y as isize);
        }
    }
}

fn fill_rect(
    framebuffer: &mut Framebuffer,
    x: isize,
    y: isize,
    width: isize,
    height: isize,
    color: u32,
) {
    if width <= 0 || height <= 0 {
        return;
    }

    framebuffer.set_current_color(color);

    for row in y..y + height {
        for column in x..x + width {
            framebuffer.point(column, row);
        }
    }
}

fn draw_rect_outline(
    framebuffer: &mut Framebuffer,
    x: isize,
    y: isize,
    width: isize,
    height: isize,
    color: u32,
) {
    if width <= 0 || height <= 0 {
        return;
    }

    let right = x + width - 1;
    let bottom = y + height - 1;

    framebuffer.set_current_color(color);
    line(framebuffer, x, y, right, y);
    line(framebuffer, right, y, right, bottom);
    line(framebuffer, right, bottom, x, bottom);
    line(framebuffer, x, bottom, x, y);
}

fn draw_text_centered(
    framebuffer: &mut Framebuffer,
    text: &str,
    y: isize,
    scale: isize,
    color: u32,
) {
    let x = (framebuffer.width as isize - text_width(text, scale)) / 2;

    draw_text(
        framebuffer,
        text,
        x + scale,
        y + scale,
        scale,
        VICTORY_SHADOW_COLOR,
    );
    draw_text(framebuffer, text, x, y, scale, color);
}

fn text_width(text: &str, scale: isize) -> isize {
    let spacing = scale;

    text.chars()
        .map(|character| {
            if character == ' ' {
                GLYPH_WIDTH / 2 * scale
            } else {
                GLYPH_WIDTH * scale
            }
        })
        .sum::<isize>()
        + text.chars().count().saturating_sub(1) as isize * spacing
}

fn draw_text(
    framebuffer: &mut Framebuffer,
    text: &str,
    x: isize,
    y: isize,
    scale: isize,
    color: u32,
) {
    let mut cursor_x = x;

    framebuffer.set_current_color(color);

    for character in text.chars() {
        if character == ' ' {
            cursor_x += GLYPH_WIDTH / 2 * scale + scale;
            continue;
        }

        draw_char(framebuffer, character, cursor_x, y, scale);
        cursor_x += GLYPH_WIDTH * scale + scale;
    }
}

fn draw_char(framebuffer: &mut Framebuffer, character: char, x: isize, y: isize, scale: isize) {
    let Some(glyph) = glyph(character) else {
        return;
    };

    for (row, bits) in glyph.iter().enumerate() {
        for column in 0..GLYPH_WIDTH {
            if bits & (1 << (GLYPH_WIDTH - 1 - column)) == 0 {
                continue;
            }

            draw_scaled_pixel(
                framebuffer,
                x + column * scale,
                y + row as isize * scale,
                scale,
            );
        }
    }
}

fn draw_scaled_pixel(framebuffer: &mut Framebuffer, x: isize, y: isize, scale: isize) {
    for dy in 0..scale {
        for dx in 0..scale {
            framebuffer.point(x + dx, y + dy);
        }
    }
}

fn glyph(character: char) -> Option<[u8; GLYPH_HEIGHT as usize]> {
    match character {
        'A' => Some([
            0b01110, 0b10001, 0b10001, 0b11111, 0b10001, 0b10001, 0b10001,
        ]),
        'C' => Some([
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ]),
        'E' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b11111,
        ]),
        'F' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        'G' => Some([
            0b01111, 0b10000, 0b10000, 0b10111, 0b10001, 0b10001, 0b01111,
        ]),
        'I' => Some([
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b11111,
        ]),
        'L' => Some([
            0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b11111,
        ]),
        'N' => Some([
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ]),
        'P' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000,
        ]),
        'R' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001,
        ]),
        'S' => Some([
            0b01111, 0b10000, 0b10000, 0b01110, 0b00001, 0b00001, 0b11110,
        ]),
        'T' => Some([
            0b11111, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100, 0b00100,
        ]),
        '0' => Some([
            0b01110, 0b10001, 0b10011, 0b10101, 0b11001, 0b10001, 0b01110,
        ]),
        '1' => Some([
            0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110,
        ]),
        '2' => Some([
            0b01110, 0b10001, 0b00001, 0b00010, 0b00100, 0b01000, 0b11111,
        ]),
        '3' => Some([
            0b11110, 0b00001, 0b00001, 0b01110, 0b00001, 0b00001, 0b11110,
        ]),
        '4' => Some([
            0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010,
        ]),
        '5' => Some([
            0b11111, 0b10000, 0b10000, 0b11110, 0b00001, 0b00001, 0b11110,
        ]),
        '6' => Some([
            0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110,
        ]),
        '7' => Some([
            0b11111, 0b00001, 0b00010, 0b00100, 0b01000, 0b01000, 0b01000,
        ]),
        '8' => Some([
            0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110,
        ]),
        '9' => Some([
            0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110,
        ]),
        ':' => Some([
            0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000,
        ]),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::f32::consts::PI;

    use crate::texture::Texture;

    use super::*;

    fn solid_texture(color: u32) -> Texture {
        Texture::new(2, 2, vec![color; 4]).expect("solid texture should be valid")
    }

    fn texture_manager_for_render_tests(
        textures: impl IntoIterator<Item = (char, u32)>,
    ) -> TextureManager {
        let textures = textures
            .into_iter()
            .map(|(wall, color)| (wall, solid_texture(color)))
            .collect::<HashMap<_, _>>();

        TextureManager::new(textures, Texture::fallback())
    }

    fn render_test_maze() -> Maze {
        vec![
            "#####".chars().collect(),
            "#   +".chars().collect(),
            "# p +".chars().collect(),
            "#   +".chars().collect(),
            "#####".chars().collect(),
        ]
    }

    fn framebuffer_contains(framebuffer: &Framebuffer, color: u32) -> bool {
        framebuffer.buffer.contains(&color)
    }

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
        assert_eq!(wall_color('&'), WALL_AMPERSAND_COLOR);
        assert_eq!(wall_color('!'), GOAL_WALL_COLOR);
        assert_ne!(wall_color('#'), wall_color('+'));
        assert_ne!(wall_color('+'), wall_color('%'));
        assert_ne!(wall_color('%'), wall_color('@'));
        assert_ne!(wall_color('@'), wall_color('&'));
        assert_ne!(wall_color('&'), wall_color('!'));
    }

    #[test]
    fn render_3d_draws_wall_pixels_from_texture() {
        let maze = render_test_maze();
        let mut player = Player::new(2, 2, 10);
        player.a = 0.0;
        let mut framebuffer = Framebuffer::new(80, 60);
        let textures = texture_manager_for_render_tests([('+', 0x123456)]);

        render_3d(&mut framebuffer, &maze, &player, 10, &textures);

        assert!(framebuffer_contains(&framebuffer, 0x123456));
    }

    #[test]
    fn render_3d_selects_texture_from_wall_impact() {
        let maze = render_test_maze();
        let textures = texture_manager_for_render_tests([('#', 0x111111), ('+', 0x222222)]);
        let mut player = Player::new(2, 2, 10);
        let mut framebuffer = Framebuffer::new(80, 60);

        player.a = 0.0;
        render_3d(&mut framebuffer, &maze, &player, 10, &textures);
        assert!(framebuffer_contains(&framebuffer, 0x222222));

        framebuffer.clear();
        player.a = PI;
        render_3d(&mut framebuffer, &maze, &player, 10, &textures);
        assert!(framebuffer_contains(&framebuffer, 0x111111));
    }

    #[test]
    fn render_3d_handles_clipped_near_wall_texture_sampling() {
        let maze = vec![
            "###".chars().collect(),
            "#p#".chars().collect(),
            "###".chars().collect(),
        ];
        let mut player = Player::new(1, 1, 10);
        player.pos.x = 19.5;
        player.pos.y = 15.0;
        player.a = 0.0;
        let mut framebuffer = Framebuffer::new(20, 20);
        let textures = texture_manager_for_render_tests([('#', 0x334455)]);

        render_3d(&mut framebuffer, &maze, &player, 10, &textures);

        assert!(framebuffer_contains(&framebuffer, 0x334455));
    }

    #[test]
    fn project_maze_minimap_dimensions_use_small_cells() {
        let maze = include_str!("../maze.txt")
            .lines()
            .map(|line| line.chars().collect())
            .collect::<Maze>();

        assert_eq!(minimap_dimensions(&maze), (120, 84));
    }

    #[test]
    fn minimap_offset_stays_in_top_right_corner() {
        let framebuffer = Framebuffer::new(800, 600);

        assert_eq!(minimap_content_offset(&framebuffer, 114, 78), (672, 14));
    }

    #[test]
    fn minimap_offset_does_not_underflow_on_small_framebuffer() {
        let framebuffer = Framebuffer::new(40, 30);

        assert_eq!(minimap_content_offset(&framebuffer, 114, 78), (4, 4));
    }

    #[test]
    fn minimap_player_position_uses_world_space() {
        let player = Player::new(1, 1, 40);

        assert_eq!(
            minimap_player_position(&player, 40, 672, 14),
            Some((681, 23))
        );
    }
}
