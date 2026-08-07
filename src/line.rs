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
