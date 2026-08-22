use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

use crate::caster::WallSide;

const DEFAULT_TEXTURE_PATHS: [(char, &str); 6] = [
    ('#', "assets/wall1.png"),
    ('+', "assets/wall2.png"),
    ('%', "assets/wall3.png"),
    ('@', "assets/wall4.png"),
    ('&', "assets/wall5.png"),
    ('!', "assets/wall5.png"),
];
const FLOOR_TEXTURE_PATH: &str = "assets/piso.png";

const FALLBACK_MAGENTA: u32 = 0xff00ff;
const FALLBACK_BLACK: u32 = 0x000000;

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pixels: Vec<u32>,
}

#[derive(Debug)]
pub enum TextureError {
    InvalidDimensions {
        width: usize,
        height: usize,
    },
    PixelCountMismatch {
        width: usize,
        height: usize,
        expected: usize,
        actual: usize,
    },
    Image(image::ImageError),
}

impl fmt::Display for TextureError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidDimensions { width, height } => {
                write!(
                    formatter,
                    "texture dimensions must be greater than zero, got {width}x{height}"
                )
            }
            Self::PixelCountMismatch {
                width,
                height,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "texture {width}x{height} needs {expected} pixels, got {actual}"
                )
            }
            Self::Image(error) => write!(formatter, "could not decode image: {error}"),
        }
    }
}

impl Error for TextureError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Image(error) => Some(error),
            _ => None,
        }
    }
}

impl Texture {
    pub fn new(width: usize, height: usize, pixels: Vec<u32>) -> Result<Self, TextureError> {
        if width == 0 || height == 0 {
            return Err(TextureError::InvalidDimensions { width, height });
        }

        let expected = width
            .checked_mul(height)
            .ok_or(TextureError::PixelCountMismatch {
                width,
                height,
                expected: usize::MAX,
                actual: pixels.len(),
            })?;

        if pixels.len() != expected {
            return Err(TextureError::PixelCountMismatch {
                width,
                height,
                expected,
                actual: pixels.len(),
            });
        }

        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, TextureError> {
        let image = image::open(path).map_err(TextureError::Image)?.to_rgba8();
        let width = image.width() as usize;
        let height = image.height() as usize;
        let pixels = image
            .pixels()
            .map(|pixel| rgb_to_u32(pixel[0], pixel[1], pixel[2]))
            .collect();

        Self::new(width, height, pixels)
    }

    /// Devuelve el pixel en formato 0xRRGGBB. Las coordenadas fuera de rango
    /// se ajustan al borde mas cercano para simplificar el renderer futuro.
    pub fn get_pixel(&self, x: usize, y: usize) -> u32 {
        if self.width == 0 || self.height == 0 {
            return FALLBACK_MAGENTA;
        }

        let clamped_x = x.min(self.width - 1);
        let clamped_y = y.min(self.height - 1);

        self.pixels[clamped_y * self.width + clamped_x]
    }

    pub fn sample(&self, u: f32, v: f32) -> u32 {
        if self.width == 0 || self.height == 0 || !u.is_finite() || !v.is_finite() {
            return FALLBACK_MAGENTA;
        }

        let x = u.clamp(0.0, 1.0) * self.width.saturating_sub(1) as f32;
        let y = v.clamp(0.0, 1.0) * self.height.saturating_sub(1) as f32;
        let x0 = x.floor() as usize;
        let y0 = y.floor() as usize;
        let x1 = (x0 + 1).min(self.width - 1);
        let y1 = (y0 + 1).min(self.height - 1);
        let x_weight = x - x0 as f32;
        let y_weight = y - y0 as f32;

        let top = blend_color(self.get_pixel(x0, y0), self.get_pixel(x1, y0), x_weight);
        let bottom = blend_color(self.get_pixel(x0, y1), self.get_pixel(x1, y1), x_weight);

        blend_color(top, bottom, y_weight)
    }

    pub fn sample_repeating_nearest(&self, u: f32, v: f32) -> u32 {
        if self.width == 0 || self.height == 0 || !u.is_finite() || !v.is_finite() {
            return FALLBACK_MAGENTA;
        }

        let x = (u.rem_euclid(1.0) * self.width as f32).floor() as usize;
        let y = (v.rem_euclid(1.0) * self.height as f32).floor() as usize;

        self.get_pixel(x.min(self.width - 1), y.min(self.height - 1))
    }

    pub fn pixel_count(&self) -> usize {
        self.pixels.len()
    }

    pub fn fallback() -> Self {
        Self {
            width: 2,
            height: 2,
            pixels: vec![
                FALLBACK_MAGENTA,
                FALLBACK_BLACK,
                FALLBACK_BLACK,
                FALLBACK_MAGENTA,
            ],
        }
    }
}

/// Calcula la columna horizontal de textura para un impacto de rayo.
/// Retorna 0 para entradas invalidas y evita panics en el renderer futuro.
pub fn texture_x_from_hit(
    hit_x: f32,
    hit_y: f32,
    side: WallSide,
    block_size: usize,
    texture_width: usize,
) -> usize {
    if texture_width == 0 {
        return 0;
    }

    let u = texture_u_from_hit(hit_x, hit_y, side, block_size);
    let tx = (u * texture_width as f32).floor() as usize;

    tx.min(texture_width - 1)
}

pub fn texture_u_from_hit(hit_x: f32, hit_y: f32, side: WallSide, block_size: usize) -> f32 {
    if block_size == 0 || !hit_x.is_finite() || !hit_y.is_finite() {
        return 0.0;
    }

    let cell_size = block_size as f32;
    let local = match side {
        WallSide::Vertical => hit_y.rem_euclid(cell_size),
        WallSide::Horizontal => hit_x.rem_euclid(cell_size),
    };

    (local / cell_size).clamp(0.0, 1.0)
}

/// Calcula la fila vertical de textura para un pixel visible de una stake.
/// Usa la stake original sin clipping. Retorna 0 para geometria invalida.
pub fn texture_y_for_stake(
    screen_y: isize,
    unclipped_stake_top: f32,
    unclipped_stake_bottom: f32,
    texture_height: usize,
) -> usize {
    if texture_height == 0
        || !unclipped_stake_top.is_finite()
        || !unclipped_stake_bottom.is_finite()
        || unclipped_stake_bottom <= unclipped_stake_top
    {
        return 0;
    }

    let v = texture_v_for_stake(screen_y, unclipped_stake_top, unclipped_stake_bottom);
    let ty = (v * texture_height as f32).floor() as usize;

    ty.min(texture_height - 1)
}

pub fn texture_v_for_stake(
    screen_y: isize,
    unclipped_stake_top: f32,
    unclipped_stake_bottom: f32,
) -> f32 {
    if !unclipped_stake_top.is_finite()
        || !unclipped_stake_bottom.is_finite()
        || unclipped_stake_bottom <= unclipped_stake_top
    {
        return 0.0;
    }

    let stake_height = unclipped_stake_bottom - unclipped_stake_top;

    ((screen_y as f32 - unclipped_stake_top) / stake_height).clamp(0.0, 1.0)
}

pub struct TextureManager {
    textures: HashMap<char, Texture>,
    fallback: Texture,
    floor: Texture,
}

impl TextureManager {
    pub fn new(textures: HashMap<char, Texture>, fallback: Texture) -> Self {
        Self {
            textures,
            floor: fallback.clone(),
            fallback,
        }
    }

    pub fn new_with_floor(
        textures: HashMap<char, Texture>,
        fallback: Texture,
        floor: Texture,
    ) -> Self {
        Self {
            textures,
            fallback,
            floor,
        }
    }

    pub fn load_default() -> Self {
        let fallback = Texture::fallback();
        let mut textures = HashMap::new();

        for (wall, path) in DEFAULT_TEXTURE_PATHS {
            if let Ok(texture) = Texture::from_file(path) {
                textures.insert(wall, texture);
            }
        }

        let floor = Texture::from_file(FLOOR_TEXTURE_PATH).unwrap_or_else(|_| fallback.clone());

        Self {
            textures,
            fallback,
            floor,
        }
    }

    pub fn get(&self, wall: char) -> &Texture {
        self.textures.get(&wall).unwrap_or(&self.fallback)
    }

    pub fn floor(&self) -> &Texture {
        &self.floor
    }

    pub fn len(&self) -> usize {
        self.textures.len()
    }

    pub fn is_empty(&self) -> bool {
        self.textures.is_empty()
    }
}

// El framebuffer del proyecto usa 0xRRGGBB; alpha se ignora por ahora.
fn rgb_to_u32(r: u8, g: u8, b: u8) -> u32 {
    ((r as u32) << 16) | ((g as u32) << 8) | b as u32
}

fn blend_color(left: u32, right: u32, weight: f32) -> u32 {
    let weight = weight.clamp(0.0, 1.0);
    let inverse = 1.0 - weight;
    let red = (((left >> 16) & 0xff) as f32 * inverse + ((right >> 16) & 0xff) as f32 * weight)
        .round() as u32;
    let green = (((left >> 8) & 0xff) as f32 * inverse + ((right >> 8) & 0xff) as f32 * weight)
        .round() as u32;
    let blue = ((left & 0xff) as f32 * inverse + (right & 0xff) as f32 * weight).round() as u32;

    (red << 16) | (green << 8) | blue
}

#[cfg(test)]
mod tests {
    use crate::caster::{Intersect, cast_ray};
    use crate::framebuffer::Framebuffer;
    use crate::maze::Maze;
    use crate::player::Player;

    use super::*;

    fn sample_texture() -> Texture {
        Texture::new(2, 2, vec![0x010203, 0x040506, 0x070809, 0x0a0b0c])
            .expect("sample texture should be valid")
    }

    fn texture_manager_for_tests() -> TextureManager {
        let mut textures = HashMap::new();
        textures.insert('#', Texture::new(1, 1, vec![0x111111]).unwrap());
        textures.insert('+', Texture::new(1, 1, vec![0x222222]).unwrap());
        textures.insert('%', Texture::new(1, 1, vec![0x333333]).unwrap());
        textures.insert('@', Texture::new(1, 1, vec![0x444444]).unwrap());
        textures.insert('&', Texture::new(1, 1, vec![0x555555]).unwrap());
        textures.insert('!', Texture::new(1, 1, vec![0x666666]).unwrap());

        TextureManager::new(textures, Texture::fallback())
    }

    fn texture_manager_with_floor_for_tests() -> TextureManager {
        TextureManager::new_with_floor(
            HashMap::new(),
            Texture::fallback(),
            Texture::new(1, 1, vec![0x123456]).unwrap(),
        )
    }

    fn raycast_test_maze() -> Maze {
        vec![
            "#####".chars().collect(),
            "#   #".chars().collect(),
            "# p #".chars().collect(),
            "#   #".chars().collect(),
            "#####".chars().collect(),
        ]
    }

    #[test]
    fn creates_valid_texture_from_memory() {
        let texture = sample_texture();

        assert_eq!(texture.width, 2);
        assert_eq!(texture.height, 2);
        assert_eq!(texture.pixel_count(), 4);
    }

    #[test]
    fn rejects_invalid_dimensions() {
        assert!(Texture::new(0, 2, vec![0x000000, 0x000000]).is_err());
        assert!(Texture::new(2, 0, vec![0x000000, 0x000000]).is_err());
    }

    #[test]
    fn rejects_wrong_pixel_count() {
        assert!(Texture::new(2, 2, vec![0x000000, 0x111111, 0x222222]).is_err());
    }

    #[test]
    fn gets_individual_pixels() {
        let texture = sample_texture();

        assert_eq!(texture.get_pixel(0, 0), 0x010203);
        assert_eq!(texture.get_pixel(1, 0), 0x040506);
        assert_eq!(texture.get_pixel(0, 1), 0x070809);
        assert_eq!(texture.get_pixel(1, 1), 0x0a0b0c);
    }

    #[test]
    fn clamps_pixel_coordinates_outside_texture() {
        let texture = sample_texture();

        assert_eq!(texture.get_pixel(8, 0), 0x040506);
        assert_eq!(texture.get_pixel(0, 8), 0x070809);
        assert_eq!(texture.get_pixel(8, 8), 0x0a0b0c);
    }

    #[test]
    fn sample_returns_exact_corner_pixels() {
        let texture = sample_texture();

        assert_eq!(texture.sample(0.0, 0.0), 0x010203);
        assert_eq!(texture.sample(1.0, 0.0), 0x040506);
        assert_eq!(texture.sample(0.0, 1.0), 0x070809);
        assert_eq!(texture.sample(1.0, 1.0), 0x0a0b0c);
    }

    #[test]
    fn sample_blends_between_neighbor_pixels() {
        let texture =
            Texture::new(2, 1, vec![0x000000, 0xffffff]).expect("sample texture should be valid");

        assert_eq!(texture.sample(0.5, 0.0), 0x808080);
    }

    #[test]
    fn sample_clamps_normalized_coordinates() {
        let texture = sample_texture();

        assert_eq!(texture.sample(-1.0, 0.0), 0x010203);
        assert_eq!(texture.sample(2.0, 1.0), 0x0a0b0c);
        assert_eq!(texture.sample(f32::NAN, 0.0), FALLBACK_MAGENTA);
    }

    #[test]
    fn sample_repeating_nearest_wraps_normalized_coordinates() {
        let texture =
            Texture::new(2, 1, vec![0x111111, 0x222222]).expect("test texture should be valid");

        assert_eq!(texture.sample_repeating_nearest(0.25, 0.0), 0x111111);
        assert_eq!(texture.sample_repeating_nearest(0.75, 0.0), 0x222222);
        assert_eq!(texture.sample_repeating_nearest(1.25, 0.0), 0x111111);
        assert_eq!(texture.sample_repeating_nearest(-0.25, 0.0), 0x222222);
    }

    #[test]
    fn creates_valid_fallback_texture() {
        let texture = Texture::fallback();

        assert_eq!(texture.width, 2);
        assert_eq!(texture.height, 2);
        assert_eq!(texture.pixel_count(), 4);
        assert_eq!(texture.get_pixel(0, 0), FALLBACK_MAGENTA);
        assert_eq!(texture.get_pixel(1, 0), FALLBACK_BLACK);
        assert_eq!(texture.get_pixel(0, 1), FALLBACK_BLACK);
        assert_eq!(texture.get_pixel(1, 1), FALLBACK_MAGENTA);
    }

    #[test]
    fn loads_placeholder_png_from_assets() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/wall1.png");
        let texture = Texture::from_file(path).expect("wall1.png should be a valid PNG texture");

        assert!(texture.width > 0);
        assert!(texture.height > 0);
        assert_eq!(texture.pixel_count(), texture.width * texture.height);
    }

    #[test]
    fn loads_goal_png_from_assets() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/wall5.png");
        let texture = Texture::from_file(path).expect("wall5.png should be a valid PNG texture");

        assert!(texture.width > 0);
        assert!(texture.height > 0);
        assert_eq!(texture.pixel_count(), texture.width * texture.height);
    }

    #[test]
    fn loads_floor_png_from_assets() {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/piso.png");
        let texture = Texture::from_file(path).expect("piso.png should be a valid PNG texture");

        assert!(texture.width > 0);
        assert!(texture.height > 0);
        assert_eq!(texture.pixel_count(), texture.width * texture.height);
    }

    #[test]
    fn default_manager_maps_goal_wall_texture() {
        let manager = TextureManager::load_default();

        assert!(!std::ptr::eq(manager.get('!'), &manager.fallback));
        assert!(!std::ptr::eq(manager.get('&'), &manager.fallback));
    }

    #[test]
    fn default_manager_loads_floor_texture() {
        let manager = TextureManager::load_default();

        assert!(!std::ptr::eq(manager.floor(), &manager.fallback));
    }

    #[test]
    fn texture_manager_maps_known_wall_characters() {
        let manager = texture_manager_for_tests();

        assert_eq!(manager.get('#').get_pixel(0, 0), 0x111111);
        assert_eq!(manager.get('+').get_pixel(0, 0), 0x222222);
        assert_eq!(manager.get('%').get_pixel(0, 0), 0x333333);
        assert_eq!(manager.get('@').get_pixel(0, 0), 0x444444);
        assert_eq!(manager.get('&').get_pixel(0, 0), 0x555555);
        assert_eq!(manager.get('!').get_pixel(0, 0), 0x666666);
    }

    #[test]
    fn texture_manager_returns_fallback_for_unknown_wall_character() {
        let manager = texture_manager_for_tests();

        assert!(std::ptr::eq(manager.get('~'), &manager.fallback));
        assert_eq!(manager.get('~').get_pixel(0, 0), FALLBACK_MAGENTA);
    }

    #[test]
    fn texture_manager_get_returns_references_without_cloning() {
        let manager = texture_manager_for_tests();
        let stored_texture = manager.textures.get(&'#').unwrap();

        assert!(std::ptr::eq(manager.get('#'), stored_texture));
        assert!(std::ptr::eq(manager.get('#'), manager.get('#')));
    }

    #[test]
    fn texture_manager_returns_configured_floor_texture() {
        let manager = texture_manager_with_floor_for_tests();

        assert_eq!(manager.floor().get_pixel(0, 0), 0x123456);
    }

    #[test]
    fn texture_x_for_vertical_wall_uses_hit_y() {
        let tx = texture_x_from_hit(80.0, 10.0, WallSide::Vertical, 40, 32);

        assert_eq!(tx, 8);
    }

    #[test]
    fn texture_u_for_vertical_wall_uses_hit_y() {
        let u = texture_u_from_hit(80.0, 10.0, WallSide::Vertical, 40);

        assert_eq!(u, 0.25);
    }

    #[test]
    fn texture_x_for_horizontal_wall_uses_hit_x() {
        let tx = texture_x_from_hit(30.0, 80.0, WallSide::Horizontal, 40, 32);

        assert_eq!(tx, 24);
    }

    #[test]
    fn texture_u_for_horizontal_wall_uses_hit_x() {
        let u = texture_u_from_hit(30.0, 80.0, WallSide::Horizontal, 40);

        assert_eq!(u, 0.75);
    }

    #[test]
    fn texture_x_repeats_for_each_cell() {
        let first = texture_x_from_hit(80.0, 10.0, WallSide::Vertical, 40, 32);
        let second = texture_x_from_hit(80.0, 50.0, WallSide::Vertical, 40, 32);
        let third = texture_x_from_hit(80.0, 90.0, WallSide::Vertical, 40, 32);

        assert_eq!(first, second);
        assert_eq!(second, third);
        assert_eq!(first, 8);
    }

    #[test]
    fn texture_x_clamps_near_end_of_texture() {
        let tx = texture_x_from_hit(80.0, 39.999, WallSide::Vertical, 40, 32);

        assert_eq!(tx, 31);
    }

    #[test]
    fn texture_x_at_cell_start_is_zero() {
        let vertical = texture_x_from_hit(80.0, 0.0, WallSide::Vertical, 40, 32);
        let horizontal = texture_x_from_hit(0.0, 80.0, WallSide::Horizontal, 40, 32);

        assert_eq!(vertical, 0);
        assert_eq!(horizontal, 0);
    }

    #[test]
    fn texture_x_returns_zero_for_invalid_inputs() {
        assert_eq!(texture_x_from_hit(10.0, 10.0, WallSide::Vertical, 0, 32), 0);
        assert_eq!(texture_x_from_hit(10.0, 10.0, WallSide::Vertical, 40, 0), 0);
        assert_eq!(
            texture_x_from_hit(f32::NAN, 10.0, WallSide::Horizontal, 40, 32),
            0
        );
        assert_eq!(
            texture_x_from_hit(10.0, f32::INFINITY, WallSide::Vertical, 40, 32),
            0
        );
    }

    #[test]
    fn texture_x_accepts_texture_width_from_texture() {
        let texture =
            Texture::new(32, 16, vec![0x000000; 32 * 16]).expect("test texture should be valid");
        let tx = texture_x_from_hit(30.0, 80.0, WallSide::Horizontal, 40, texture.width);

        assert_eq!(tx, 24);
    }

    #[test]
    fn texture_x_works_with_cast_ray_intersection() {
        let maze = raycast_test_maze();
        let player = Player::new(2, 2, 10);
        let mut framebuffer = Framebuffer::new(80, 80);
        let texture_width = 32;

        let intersection = cast_ray(&mut framebuffer, &maze, &player, 0.0, 10, 0, 0, false);
        let tx = texture_x_from_hit(
            intersection.hit_x,
            intersection.hit_y,
            intersection.side,
            10,
            texture_width,
        );

        assert!(tx < texture_width);
    }

    #[test]
    fn texture_coordinates_sample_expected_pixel_from_intersection() {
        let intersection = Intersect {
            distance: 10.0,
            impact: '#',
            hit_x: 80.0,
            hit_y: 10.0,
            side: WallSide::Vertical,
        };
        let mut pixels = vec![0x000000; 32 * 32];
        pixels[16 * 32 + 8] = 0xabcdef;
        let texture = Texture::new(32, 32, pixels).expect("test texture should be valid");

        let tx = texture_x_from_hit(
            intersection.hit_x,
            intersection.hit_y,
            intersection.side,
            40,
            texture.width,
        );
        let ty = texture_y_for_stake(200, 100.0, 300.0, texture.height);

        assert_eq!(texture.get_pixel(tx, ty), 0xabcdef);
    }

    #[test]
    fn texture_y_at_unclipped_stake_top_is_zero() {
        let ty = texture_y_for_stake(100, 100.0, 300.0, 32);

        assert_eq!(ty, 0);
    }

    #[test]
    fn texture_y_at_unclipped_stake_center_is_middle_row() {
        let ty = texture_y_for_stake(200, 100.0, 300.0, 32);

        assert_eq!(ty, 16);
    }

    #[test]
    fn texture_v_at_unclipped_stake_center_is_half() {
        let v = texture_v_for_stake(200, 100.0, 300.0);

        assert_eq!(v, 0.5);
    }

    #[test]
    fn texture_y_clamps_near_unclipped_stake_bottom() {
        let ty = texture_y_for_stake(300, 100.0, 300.0, 32);

        assert_eq!(ty, 31);
    }

    #[test]
    fn texture_y_uses_unclipped_stake_bounds_after_clipping() {
        let ty = texture_y_for_stake(0, -200.0, 800.0, 100);

        assert_eq!(ty, 20);
    }

    #[test]
    fn texture_y_clamps_screen_y_outside_valid_stake() {
        let before_top = texture_y_for_stake(50, 100.0, 300.0, 32);
        let after_bottom = texture_y_for_stake(350, 100.0, 300.0, 32);

        assert_eq!(before_top, 0);
        assert_eq!(after_bottom, 31);
    }

    #[test]
    fn texture_y_returns_zero_for_invalid_inputs() {
        assert_eq!(texture_y_for_stake(100, 100.0, 300.0, 0), 0);
        assert_eq!(texture_y_for_stake(100, 100.0, 100.0, 32), 0);
        assert_eq!(texture_y_for_stake(100, 300.0, 100.0, 32), 0);
        assert_eq!(texture_y_for_stake(100, f32::NAN, 300.0, 32), 0);
        assert_eq!(texture_y_for_stake(100, 100.0, f32::INFINITY, 32), 0);
    }
}
