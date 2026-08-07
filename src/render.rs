use crate::framebuffer::Framebuffer;
use crate::maze::Maze;

const WALL_COLOR: u32 = 0x1f2937;
const PATH_COLOR: u32 = 0xd1d5db;
const PLAYER_START_COLOR: u32 = 0x22c55e;
const GOAL_COLOR: u32 = 0xfacc15;
const UNKNOWN_COLOR: u32 = 0xff00ff;

pub fn render_maze(framebuffer: &mut Framebuffer, maze: &Maze, block_size: usize) {
    if maze.is_empty() || block_size == 0 {
        return;
    }

    let maze_width = maze.iter().map(|row| row.len()).max().unwrap_or(0) * block_size;
    let maze_height = maze.len() * block_size;
    let offset_x = framebuffer.width.saturating_sub(maze_width) / 2;
    let offset_y = framebuffer.height.saturating_sub(maze_height) / 2;

    for (row_index, row) in maze.iter().enumerate() {
        for (column_index, cell) in row.iter().enumerate() {
            let x0 = offset_x + column_index * block_size;
            let y0 = offset_y + row_index * block_size;

            draw_cell(framebuffer, x0, y0, block_size, *cell);
        }
    }
}

fn draw_cell(framebuffer: &mut Framebuffer, x0: usize, y0: usize, block_size: usize, cell: char) {
    let color = match cell {
        '#' => WALL_COLOR,
        ' ' => PATH_COLOR,
        'p' => PLAYER_START_COLOR,
        'g' => GOAL_COLOR,
        _ => UNKNOWN_COLOR,
    };

    framebuffer.set_current_color(color);

    // Dejamos una línea de fondo entre celdas para inspeccionar mejor el mapa.
    let visible_size = block_size.saturating_sub(1);

    for y in 0..visible_size {
        for x in 0..visible_size {
            framebuffer.point((x0 + x) as isize, (y0 + y) as isize);
        }
    }
}
