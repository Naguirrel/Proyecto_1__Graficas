use crate::framebuffer::Framebuffer;

pub fn line(framebuffer: &mut Framebuffer, x0: isize, y0: isize, x1: isize, y1: isize) {
    let mut x = x0;
    let mut y = y0;

    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut error = dx + dy;

    loop {
        framebuffer.point(x, y);

        if x == x1 && y == y1 {
            break;
        }

        // Bresenham decide si avanzar en x, en y, o en ambos.
        let double_error = 2 * error;

        if double_error >= dy {
            error += dy;
            x += sx;
        }

        if double_error <= dx {
            error += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pixel_is_set(framebuffer: &Framebuffer, x: usize, y: usize) -> bool {
        framebuffer.buffer[y * framebuffer.width + x] == framebuffer.current_color
    }

    #[test]
    fn draws_horizontal_vertical_and_single_point_lines() {
        let mut framebuffer = Framebuffer::new(8, 8);

        line(&mut framebuffer, 1, 3, 5, 3);
        for x in 1..=5 {
            assert!(pixel_is_set(&framebuffer, x, 3));
        }

        framebuffer.clear();
        line(&mut framebuffer, 4, 6, 4, 2);
        for y in 2..=6 {
            assert!(pixel_is_set(&framebuffer, 4, y));
        }

        framebuffer.clear();
        line(&mut framebuffer, 2, 2, 2, 2);
        assert!(pixel_is_set(&framebuffer, 2, 2));
    }

    #[test]
    fn draws_diagonal_lines_in_both_directions() {
        let mut framebuffer = Framebuffer::new(8, 8);

        line(&mut framebuffer, 1, 1, 5, 5);
        for point in [(1, 1), (2, 2), (3, 3), (4, 4), (5, 5)] {
            assert!(pixel_is_set(&framebuffer, point.0, point.1));
        }

        framebuffer.clear();
        line(&mut framebuffer, 5, 1, 1, 5);
        for point in [(5, 1), (4, 2), (3, 3), (2, 4), (1, 5)] {
            assert!(pixel_is_set(&framebuffer, point.0, point.1));
        }
    }

    #[test]
    fn clipped_lines_do_not_panic_and_draw_visible_pixels() {
        let mut framebuffer = Framebuffer::new(5, 5);

        line(&mut framebuffer, -3, 2, 3, 2);

        for x in 0..=3 {
            assert!(pixel_is_set(&framebuffer, x, 2));
        }
    }
}
