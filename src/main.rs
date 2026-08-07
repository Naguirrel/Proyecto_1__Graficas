mod framebuffer;
mod line;

use framebuffer::Framebuffer;
use line::line;
use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 800;
const HEIGHT: usize = 600;

fn draw_bresenham_demo(framebuffer: &mut Framebuffer) {
    let center_x = (WIDTH / 2) as isize;
    let center_y = (HEIGHT / 2) as isize;

    framebuffer.set_current_color(0xff3333);
    line(framebuffer, center_x, center_y, 700, center_y);

    framebuffer.set_current_color(0x33ff66);
    line(framebuffer, center_x, center_y, center_x, 80);

    framebuffer.set_current_color(0x3388ff);
    line(framebuffer, center_x, center_y, 650, 100);

    framebuffer.set_current_color(0xffcc33);
    line(framebuffer, center_x, center_y, 650, 500);

    framebuffer.set_current_color(0xff66ff);
    line(framebuffer, 120, 500, center_x, center_y);

    framebuffer.set_current_color(0xffffff);
    line(framebuffer, 50, 50, 760, 560);
}

fn main() -> Result<(), minifb::Error> {
    let mut window = Window::new(
        "Proyecto 1 - Raycasting",
        WIDTH,
        HEIGHT,
        WindowOptions::default(),
    )?;

    let mut framebuffer = Framebuffer::new(WIDTH, HEIGHT);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        framebuffer.clear();
        draw_bresenham_demo(&mut framebuffer);

        window.update_with_buffer(&framebuffer.buffer, WIDTH, HEIGHT)?;
    }

    Ok(())
}
