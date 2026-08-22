use std::f32::consts::PI;

use crate::caster::cast_ray;
use crate::framebuffer::Framebuffer;
use crate::line::line;
use crate::maze::Maze;
use crate::player::Player;
use crate::sprite::{SpriteKind, WorldSprite};
use crate::texture::{Texture, TextureManager, texture_u_from_hit};

const WALL_COLOR: u32 = 0x3b82f6;
const WALL_PLUS_COLOR: u32 = 0xef4444;
const WALL_PERCENT_COLOR: u32 = 0x22c55e;
const WALL_AT_COLOR: u32 = 0xa855f7;
const WALL_AMPERSAND_COLOR: u32 = 0xf97316;
const GOAL_WALL_COLOR: u32 = 0xff00ff;
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
const MENU_MAZE_CELL_SIZE: usize = 10;
const MENU_MAZE_BOTTOM_MARGIN: usize = 30;
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
const HORIZON_EPSILON: f32 = 0.0001;
const SPRITE_FOV_MARGIN: f32 = 0.35;

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
    render_3d_with_sprites(framebuffer, maze, player, block_size, textures, &[]);
}

pub fn render_3d_with_sprites(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    player: &Player,
    block_size: usize,
    textures: &TextureManager,
    sprites: &[WorldSprite],
) {
    if framebuffer.width == 0 || framebuffer.height == 0 {
        return;
    }

    let projection_plane = distance_to_projection_plane(framebuffer.width, player.fov);
    let screen_center = framebuffer.height as f32 / 2.0 + player.height * JUMP_VISUAL_SCALE;
    let last_column = framebuffer.width.saturating_sub(1);
    let mut z_buffer = vec![f32::INFINITY; framebuffer.width];

    render_3d_background(
        framebuffer,
        player,
        block_size,
        projection_plane,
        screen_center,
        textures.floor(),
    );

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
        z_buffer[x] = corrected_distance;
        let stake_height =
            projected_stake_height(block_size as f32, corrected_distance, projection_plane);
        let unclipped_stake_top = screen_center - stake_height / 2.0;
        let unclipped_stake_bottom = screen_center + stake_height / 2.0;
        let clipped_top = unclipped_stake_top.max(0.0).round() as isize;
        let clipped_bottom = unclipped_stake_bottom
            .min((framebuffer.height - 1) as f32)
            .round() as isize;
        let texture = textures.get(intersection.impact);
        let u = texture_u_from_hit(
            intersection.hit_x,
            intersection.hit_y,
            intersection.side,
            block_size,
        );

        draw_textured_wall_column(
            framebuffer,
            x,
            clipped_top,
            clipped_bottom,
            unclipped_stake_top,
            unclipped_stake_bottom,
            texture,
            u,
        );
    }

    render_world_sprites(
        framebuffer,
        player,
        block_size,
        projection_plane,
        screen_center,
        textures,
        sprites,
        &z_buffer,
    );
}

pub fn render_welcome_screen(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    selected_level_index: usize,
    level_count: usize,
    selected_option_index: usize,
) {
    let level_text = format!("NIVEL {} DE {}", selected_level_index + 1, level_count);

    render_welcome_menu_screen(
        framebuffer,
        maze,
        "RAYCASTING",
        Some(&level_text),
        VICTORY_ACCENT_COLOR,
        7,
        20,
    );
    draw_menu_options(
        framebuffer,
        &["INICIAR", "CAMBIAR NIVEL", "SALIR"],
        selected_option_index,
        325,
        4,
    );
}

fn render_welcome_menu_screen(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    title: &str,
    detail: Option<&str>,
    accent_color: u32,
    motif_cell_size: usize,
    motif_bottom_margin: usize,
) {
    render_menu_frame(
        framebuffer,
        maze,
        accent_color,
        motif_cell_size,
        motif_bottom_margin,
    );
    draw_text_centered(framebuffer, title, 135, 8, VICTORY_TEXT_COLOR);
    if let Some(detail) = detail {
        draw_text_centered(framebuffer, detail, 245, 5, VICTORY_TEXT_COLOR);
    }
}

pub fn render_victory_screen(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    selected_level_index: usize,
    level_count: usize,
    selected_option_index: usize,
) {
    render_menu_screen(
        framebuffer,
        maze,
        "GANASTE",
        None,
        "",
        "",
        VICTORY_ACCENT_COLOR,
    );

    if selected_level_index + 1 < level_count {
        draw_menu_options(
            framebuffer,
            &["REINICIAR", "SIGUIENTE NIVEL", "MENU PRINCIPAL"],
            selected_option_index,
            285,
            5,
        );
    } else {
        draw_menu_options(
            framebuffer,
            &["REINICIAR", "MENU PRINCIPAL"],
            selected_option_index,
            310,
            5,
        );
    }
}

pub fn render_pause_menu(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    selected_level_index: usize,
    level_count: usize,
    selected_option_index: usize,
) {
    let level_text = format!("NIVEL {} DE {}", selected_level_index + 1, level_count);

    render_menu_frame(framebuffer, maze, VICTORY_ACCENT_COLOR, 7, 20);
    draw_text_centered(framebuffer, "PAUSA", 135, 8, VICTORY_TEXT_COLOR);
    draw_text_centered(framebuffer, &level_text, 235, 5, VICTORY_TEXT_COLOR);
    draw_menu_options(
        framebuffer,
        &["CONTINUAR", "CAMBIAR NIVEL", "MENU PRINCIPAL"],
        selected_option_index,
        320,
        4,
    );
}

fn render_menu_screen(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    title: &str,
    detail: Option<&str>,
    primary_action: &str,
    secondary_action: &str,
    accent_color: u32,
) {
    render_menu_screen_with_motif(
        framebuffer,
        maze,
        title,
        detail,
        primary_action,
        secondary_action,
        accent_color,
        MENU_MAZE_CELL_SIZE,
        MENU_MAZE_BOTTOM_MARGIN,
    );
}

fn render_menu_screen_with_motif(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    title: &str,
    detail: Option<&str>,
    primary_action: &str,
    secondary_action: &str,
    accent_color: u32,
    motif_cell_size: usize,
    motif_bottom_margin: usize,
) {
    render_menu_frame(
        framebuffer,
        maze,
        accent_color,
        motif_cell_size,
        motif_bottom_margin,
    );
    draw_text_centered(framebuffer, title, 135, 8, VICTORY_TEXT_COLOR);
    if let Some(detail) = detail {
        draw_text_centered(framebuffer, detail, 255, 5, VICTORY_TEXT_COLOR);
    }
    draw_text_centered(framebuffer, primary_action, 315, 5, accent_color);
    draw_text_centered(framebuffer, secondary_action, 380, 5, VICTORY_TEXT_COLOR);
}

fn render_menu_frame(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    accent_color: u32,
    motif_cell_size: usize,
    motif_bottom_margin: usize,
) {
    fill_screen(framebuffer, VICTORY_BACKGROUND_COLOR);
    draw_menu_wall_bands(framebuffer);
    draw_menu_maze_motif(framebuffer, maze, motif_cell_size, motif_bottom_margin);

    let margin = 24;
    framebuffer.set_current_color(accent_color);
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
}

fn draw_menu_options(
    framebuffer: &mut Framebuffer,
    options: &[&str],
    selected_option_index: usize,
    start_y: isize,
    scale: isize,
) {
    let row_gap = GLYPH_HEIGHT * scale + 18;

    for (index, option) in options.iter().enumerate() {
        let y = start_y + index as isize * row_gap;
        let color = if index == selected_option_index {
            VICTORY_ACCENT_COLOR
        } else {
            VICTORY_TEXT_COLOR
        };

        draw_text_centered(framebuffer, option, y, scale, color);
    }
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

fn render_3d_background(
    framebuffer: &mut Framebuffer,
    player: &Player,
    block_size: usize,
    projection_plane: f32,
    screen_center: f32,
    floor_texture: &Texture,
) {
    let horizon = screen_center.clamp(0.0, framebuffer.height as f32);
    let floor_top = horizon.floor() as usize;
    let width = framebuffer.width;
    let height = framebuffer.height;

    for y in 0..floor_top {
        let row_start = y * width;
        framebuffer.buffer[row_start..row_start + width].fill(CEILING_COLOR);
    }

    if block_size == 0 || !projection_plane.is_finite() || !screen_center.is_finite() {
        for y in floor_top..height {
            let row_start = y * width;
            framebuffer.buffer[row_start..row_start + width].fill(FLOOR_COLOR);
        }
        return;
    }

    let block_size_f = block_size as f32;
    let left_angle = player.a - player.fov / 2.0;
    let right_angle = player.a + player.fov / 2.0;
    let left_correction = (left_angle - player.a).cos().abs().max(HORIZON_EPSILON);
    let right_correction = (right_angle - player.a).cos().abs().max(HORIZON_EPSILON);
    let left_dir_x = left_angle.cos();
    let left_dir_y = left_angle.sin();
    let right_dir_x = right_angle.cos();
    let right_dir_y = right_angle.sin();
    let texture_scale_x = floor_texture.width as f32 / block_size_f;
    let texture_scale_y = floor_texture.height as f32 / block_size_f;

    for y in floor_top..height {
        let row_start = y * width;
        let row_offset = y as f32 - screen_center;

        if row_offset <= HORIZON_EPSILON {
            framebuffer.buffer[row_start..row_start + width].fill(FLOOR_COLOR);
            continue;
        }

        let base_distance = block_size_f * projection_plane / row_offset;
        let left_distance = base_distance / left_correction;
        let right_distance = base_distance / right_correction;
        let world_x = player.pos.x + left_dir_x * left_distance;
        let world_y = player.pos.y + left_dir_y * left_distance;
        let right_world_x = player.pos.x + right_dir_x * right_distance;
        let right_world_y = player.pos.y + right_dir_y * right_distance;
        let step_divisor = width.saturating_sub(1).max(1) as f32;
        let step_x = (right_world_x - world_x) / step_divisor;
        let step_y = (right_world_y - world_y) / step_divisor;
        let mut texture_x = world_x * texture_scale_x;
        let mut texture_y = world_y * texture_scale_y;
        let texture_step_x = step_x * texture_scale_x;
        let texture_step_y = step_y * texture_scale_y;

        for pixel in &mut framebuffer.buffer[row_start..row_start + width] {
            *pixel = floor_texture.get_pixel(
                wrapped_texture_index(texture_x, floor_texture.width),
                wrapped_texture_index(texture_y, floor_texture.height),
            );
            texture_x += texture_step_x;
            texture_y += texture_step_y;
        }
    }
}

fn wrapped_texture_index(value: f32, size: usize) -> usize {
    if size == 0 || !value.is_finite() {
        return 0;
    }

    (value.floor() as isize).rem_euclid(size as isize) as usize
}

fn draw_textured_wall_column(
    framebuffer: &mut Framebuffer,
    x: usize,
    clipped_top: isize,
    clipped_bottom: isize,
    unclipped_stake_top: f32,
    unclipped_stake_bottom: f32,
    texture: &Texture,
    u: f32,
) {
    if x >= framebuffer.width || clipped_bottom < clipped_top {
        return;
    }

    let y_start = clipped_top.max(0) as usize;
    let y_end = clipped_bottom.min(framebuffer.height.saturating_sub(1) as isize) as usize;
    let stake_height = unclipped_stake_bottom - unclipped_stake_top;

    if !stake_height.is_finite() || stake_height <= 0.0 {
        return;
    }

    let texture_x = clamped_texture_index(u * texture.width as f32, texture.width);
    let mut texture_y =
        ((y_start as f32 - unclipped_stake_top) / stake_height) * texture.height as f32;
    let texture_y_step = texture.height as f32 / stake_height;

    for y in y_start..=y_end {
        framebuffer.buffer[y * framebuffer.width + x] =
            texture.get_pixel(texture_x, clamped_texture_index(texture_y, texture.height));
        texture_y += texture_y_step;
    }
}

fn render_world_sprites(
    framebuffer: &mut Framebuffer,
    player: &Player,
    block_size: usize,
    projection_plane: f32,
    screen_center: f32,
    textures: &TextureManager,
    sprites: &[WorldSprite],
    z_buffer: &[f32],
) {
    let mut projected_sprites = sprites
        .iter()
        .filter(|sprite| sprite.active)
        .filter_map(|sprite| {
            SpriteProjection::from_sprite(
                sprite,
                player,
                block_size,
                projection_plane,
                screen_center,
                framebuffer.width,
                framebuffer.height,
            )
            .map(|projection| (sprite, projection))
        })
        .collect::<Vec<_>>();

    projected_sprites.sort_by(|(_, left), (_, right)| {
        right
            .distance
            .partial_cmp(&left.distance)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    for (sprite, projection) in projected_sprites {
        draw_sprite(
            framebuffer,
            textures.sprite(sprite.kind),
            &projection,
            z_buffer,
        );
    }
}

fn draw_sprite(
    framebuffer: &mut Framebuffer,
    texture: &Texture,
    projection: &SpriteProjection,
    z_buffer: &[f32],
) {
    for screen_x in projection.start_x..=projection.end_x {
        let x = screen_x as usize;

        if x >= framebuffer.width || projection.distance >= z_buffer[x] {
            continue;
        }

        let texture_x = ((screen_x - projection.start_x) as usize * texture.width
            / projection.size)
            .min(texture.width.saturating_sub(1));

        for screen_y in projection.start_y..=projection.end_y {
            let y = screen_y as usize;

            if y >= framebuffer.height {
                continue;
            }

            let texture_y = ((screen_y - projection.start_y) as usize * texture.height
                / projection.size)
                .min(texture.height.saturating_sub(1));

            if texture.is_transparent(texture_x, texture_y) {
                continue;
            }

            framebuffer.buffer[y * framebuffer.width + x] = texture.get_pixel(texture_x, texture_y);
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct SpriteProjection {
    start_x: isize,
    end_x: isize,
    start_y: isize,
    end_y: isize,
    size: usize,
    distance: f32,
}

impl SpriteProjection {
    fn from_sprite(
        sprite: &WorldSprite,
        player: &Player,
        block_size: usize,
        projection_plane: f32,
        screen_center: f32,
        screen_width: usize,
        screen_height: usize,
    ) -> Option<Self> {
        if block_size == 0 || screen_width == 0 || screen_height == 0 {
            return None;
        }

        let dx = sprite.pos.x - player.pos.x;
        let dy = sprite.pos.y - player.pos.y;
        let distance = (dx * dx + dy * dy).sqrt();

        if distance <= 0.0001 {
            return None;
        }

        let sprite_angle = dy.atan2(dx);
        let angle_diff = normalize_angle(sprite_angle - player.a);

        if angle_diff.abs() > player.fov / 2.0 + SPRITE_FOV_MARGIN {
            return None;
        }

        let corrected_distance = distance * angle_diff.cos();
        if corrected_distance <= 0.0001 {
            return None;
        }

        let scale = sprite_scale(sprite.kind);
        let size = ((block_size as f32 * projection_plane / corrected_distance) * scale)
            .round()
            .max(1.0) as usize;
        let center_x = ((angle_diff + player.fov / 2.0) / player.fov) * screen_width as f32;
        let start_x = (center_x - size as f32 / 2.0).round() as isize;
        let start_y = (screen_center - size as f32 / 2.0).round() as isize;
        let end_x = (start_x + size as isize - 1).min(screen_width as isize - 1);
        let end_y = (start_y + size as isize - 1).min(screen_height as isize - 1);
        let start_x = start_x.max(0);
        let start_y = start_y.max(0);

        if end_x < start_x || end_y < start_y {
            return None;
        }

        Some(Self {
            start_x,
            end_x,
            start_y,
            end_y,
            size,
            distance: corrected_distance,
        })
    }
}

fn normalize_angle(angle: f32) -> f32 {
    (angle + PI).rem_euclid(2.0 * PI) - PI
}

fn sprite_scale(kind: SpriteKind) -> f32 {
    match kind {
        SpriteKind::Food => 0.45,
        SpriteKind::Ghost1 => 0.9,
    }
}

fn clamped_texture_index(value: f32, size: usize) -> usize {
    if size == 0 || !value.is_finite() {
        return 0;
    }

    (value.floor() as usize).min(size - 1)
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

fn draw_menu_wall_bands(framebuffer: &mut Framebuffer) {
    let walls = ['#', '+', '%', '@', '&', '!'];

    if framebuffer.width == 0 || framebuffer.height == 0 {
        return;
    }

    let band_width = (framebuffer.width as isize / walls.len() as isize).max(1);
    let top_y = 68;
    let bottom_y = framebuffer.height as isize - 76;

    for (index, wall) in walls.iter().enumerate() {
        let x = index as isize * band_width;
        let color = scale_color(wall_color(*wall), 1, 2);

        fill_rect(framebuffer, x, top_y, band_width, 8, color);
        fill_rect(framebuffer, x, bottom_y, band_width, 8, color);
    }
}

fn draw_menu_maze_motif(
    framebuffer: &mut Framebuffer,
    maze: &Maze,
    cell_size_limit: usize,
    bottom_margin: usize,
) {
    if maze.is_empty() || framebuffer.width == 0 || framebuffer.height == 0 {
        return;
    }

    let maze_width = maze.iter().map(|row| row.len()).max().unwrap_or(0);
    let maze_height = maze.len();

    if maze_width == 0 || maze_height == 0 {
        return;
    }

    let cell_size = cell_size_limit
        .min((framebuffer.width / maze_width).max(1))
        .min((framebuffer.height / maze_height).max(1));
    let motif_width = maze_width * cell_size;
    let motif_height = maze_height * cell_size;
    let offset_x = framebuffer.width.saturating_sub(motif_width) / 2;
    let offset_y = framebuffer
        .height
        .saturating_sub(motif_height + bottom_margin);

    for (row_index, row) in maze.iter().enumerate() {
        for (column_index, cell) in row.iter().enumerate() {
            let color = match *cell {
                '#' | '+' | '%' | '@' | '&' | '!' => scale_color(wall_color(*cell), 2, 3),
                'g' => GOAL_COLOR,
                'p' => PLAYER_COLOR,
                ' ' => scale_color(PATH_COLOR, 1, 5),
                _ => UNKNOWN_COLOR,
            };
            let visible_size = cell_size.saturating_sub(1).max(1);

            fill_rect(
                framebuffer,
                (offset_x + column_index * cell_size) as isize,
                (offset_y + row_index * cell_size) as isize,
                visible_size as isize,
                visible_size as isize,
                color,
            );
        }
    }
}

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

fn scale_color(color: u32, numerator: u32, denominator: u32) -> u32 {
    if denominator == 0 {
        return color;
    }

    let red = ((color >> 16) & 0xff) * numerator / denominator;
    let green = ((color >> 8) & 0xff) * numerator / denominator;
    let blue = (color & 0xff) * numerator / denominator;

    (red << 16) | (green << 8) | blue
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
        'B' => Some([
            0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110,
        ]),
        'C' => Some([
            0b01111, 0b10000, 0b10000, 0b10000, 0b10000, 0b10000, 0b01111,
        ]),
        'D' => Some([
            0b11110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b11110,
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
        'M' => Some([
            0b10001, 0b11011, 0b10101, 0b10101, 0b10001, 0b10001, 0b10001,
        ]),
        'N' => Some([
            0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001,
        ]),
        'O' => Some([
            0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
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
        'U' => Some([
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110,
        ]),
        'V' => Some([
            0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01010, 0b00100,
        ]),
        'Y' => Some([
            0b10001, 0b10001, 0b01010, 0b00100, 0b00100, 0b00100, 0b00100,
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
    use std::time::Instant;

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

    fn texture_manager_from_textures(
        textures: impl IntoIterator<Item = (char, Texture)>,
    ) -> TextureManager {
        TextureManager::new(textures.into_iter().collect(), Texture::fallback())
    }

    fn texture_manager_with_sprite_color(
        wall: char,
        wall_color: u32,
        sprite_kind: SpriteKind,
        sprite_color: u32,
    ) -> TextureManager {
        let mut wall_textures = HashMap::new();
        wall_textures.insert(wall, solid_texture(wall_color));

        let food = if sprite_kind == SpriteKind::Food {
            solid_texture(sprite_color)
        } else {
            Texture::fallback()
        };
        let ghost1 = if sprite_kind == SpriteKind::Ghost1 {
            solid_texture(sprite_color)
        } else {
            Texture::fallback()
        };

        TextureManager::new_with_floor_and_sprites(
            wall_textures,
            Texture::fallback(),
            Texture::fallback(),
            food,
            ghost1,
        )
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
    fn render_3d_keeps_ceiling_solid() {
        let maze = render_test_maze();
        let mut player = Player::new(2, 2, 10);
        player.a = 0.0;
        let mut framebuffer = Framebuffer::new(80, 60);
        let textures = texture_manager_for_render_tests([('#', 0x445566), ('+', 0x123456)]);

        render_3d(&mut framebuffer, &maze, &player, 10, &textures);

        assert_eq!(framebuffer.buffer[0], CEILING_COLOR);
    }

    #[test]
    fn render_3d_draws_floor_from_floor_texture() {
        let maze = render_test_maze();
        let mut player = Player::new(2, 2, 10);
        player.a = 0.0;
        let mut framebuffer = Framebuffer::new(80, 60);
        let mut wall_textures = HashMap::new();
        wall_textures.insert('+', solid_texture(0x123456));
        let textures = TextureManager::new_with_floor(
            wall_textures,
            Texture::fallback(),
            solid_texture(0x445566),
        );

        render_3d(&mut framebuffer, &maze, &player, 10, &textures);

        assert_eq!(framebuffer.buffer[framebuffer.buffer.len() - 1], 0x445566);
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
    fn render_3d_uses_direct_texture_column_samples() {
        let maze = vec![
            "###".chars().collect(),
            "#p#".chars().collect(),
            "###".chars().collect(),
        ];
        let mut player = Player::new(1, 1, 10);
        player.a = 0.0;
        let mut framebuffer = Framebuffer::new(1, 20);
        let gradient =
            Texture::new(2, 1, vec![0x000000, 0xffffff]).expect("test texture should be valid");
        let textures = texture_manager_from_textures([('#', gradient)]);

        render_3d(&mut framebuffer, &maze, &player, 10, &textures);

        assert!(framebuffer_contains(&framebuffer, 0xffffff));
    }

    #[test]
    fn sprite_projection_places_sprite_in_front_near_center() {
        let mut player = Player::new(1, 1, 10);
        player.a = 0.0;
        let sprite = WorldSprite::new(SpriteKind::Food, 2, 1, 10);
        let projection = SpriteProjection::from_sprite(
            &sprite,
            &player,
            10,
            distance_to_projection_plane(80, player.fov),
            30.0,
            80,
            60,
        )
        .expect("sprite in front should project");

        assert!(projection.start_x < 40);
        assert!(projection.end_x > 40);
    }

    #[test]
    fn sprite_projection_rejects_sprite_behind_player() {
        let mut player = Player::new(1, 1, 10);
        player.a = 0.0;
        let sprite = WorldSprite::new(SpriteKind::Food, 0, 1, 10);

        assert!(
            SpriteProjection::from_sprite(
                &sprite,
                &player,
                10,
                distance_to_projection_plane(80, player.fov),
                30.0,
                80,
                60,
            )
            .is_none()
        );
    }

    #[test]
    fn render_3d_draws_visible_sprite_in_front_of_wall() {
        let maze = vec![
            "#####".chars().collect(),
            "#p  #".chars().collect(),
            "#####".chars().collect(),
        ];
        let mut player = Player::new(1, 1, 10);
        player.a = 0.0;
        let sprite = WorldSprite::new(SpriteKind::Food, 2, 1, 10);
        let mut framebuffer = Framebuffer::new(80, 60);
        let textures = texture_manager_with_sprite_color('#', 0x111111, SpriteKind::Food, 0xabcdef);

        render_3d_with_sprites(&mut framebuffer, &maze, &player, 10, &textures, &[sprite]);

        assert!(framebuffer_contains(&framebuffer, 0xabcdef));
    }

    #[test]
    #[ignore]
    fn render_3d_full_resolution_benchmark() {
        let maze = render_test_maze();
        let mut player = Player::new(2, 2, 10);
        let mut framebuffer = Framebuffer::new(800, 600);
        let textures = TextureManager::load_default();
        let frames = 120;
        let start = Instant::now();

        for frame in 0..frames {
            player.a = frame as f32 * 0.01;
            render_3d(&mut framebuffer, &maze, &player, 10, &textures);
        }

        let elapsed = start.elapsed().as_secs_f64();
        let fps = frames as f64 / elapsed;
        println!(
            "render_3d 800x600: {fps:.1} FPS ({:.2} ms/frame)",
            1000.0 / fps
        );

        assert!(framebuffer.buffer.iter().any(|pixel| *pixel != 0));
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

    #[test]
    fn welcome_screen_draws_title_and_level_motif() {
        let maze = render_test_maze();
        let mut framebuffer = Framebuffer::new(800, 600);

        render_welcome_screen(&mut framebuffer, &maze, 1, 3, 0);

        assert!(framebuffer_contains(&framebuffer, VICTORY_TEXT_COLOR));
        assert!(framebuffer_contains(
            &framebuffer,
            scale_color(wall_color('#'), 2, 3)
        ));
        assert!(glyph('V').is_some());
        assert!(glyph('Y').is_some());
    }

    #[test]
    fn victory_screen_uses_same_menu_background_as_welcome() {
        let maze = render_test_maze();
        let mut welcome = Framebuffer::new(800, 600);
        let mut victory = Framebuffer::new(800, 600);

        render_welcome_screen(&mut welcome, &maze, 0, 3, 0);
        render_victory_screen(&mut victory, &maze, 0, 3, 0);

        assert_eq!(welcome.buffer[0], victory.buffer[0]);
        assert!(framebuffer_contains(&victory, VICTORY_ACCENT_COLOR));
    }

    #[test]
    fn victory_screen_can_render_without_next_level_option() {
        let maze = render_test_maze();
        let mut framebuffer = Framebuffer::new(800, 600);

        render_victory_screen(&mut framebuffer, &maze, 2, 3, 0);

        assert!(framebuffer_contains(&framebuffer, VICTORY_ACCENT_COLOR));
        assert!(glyph('M').is_some());
        assert!(glyph('U').is_some());
    }

    #[test]
    fn pause_menu_draws_all_requested_options() {
        let maze = render_test_maze();
        let mut framebuffer = Framebuffer::new(800, 600);

        render_pause_menu(&mut framebuffer, &maze, 1, 3, 0);

        assert!(framebuffer_contains(&framebuffer, VICTORY_ACCENT_COLOR));
        assert!(glyph('O').is_some());
    }
}
