use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::path::Path;

const DEFAULT_TEXTURE_PATHS: [(char, &str); 4] = [
    ('#', "assets/wall1.png"),
    ('+', "assets/wall2.png"),
    ('%', "assets/wall3.png"),
    ('@', "assets/wall4.png"),
];

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

pub struct TextureManager {
    textures: HashMap<char, Texture>,
    fallback: Texture,
}

impl TextureManager {
    pub fn new(textures: HashMap<char, Texture>, fallback: Texture) -> Self {
        Self { textures, fallback }
    }

    pub fn load_default() -> Self {
        let fallback = Texture::fallback();
        let mut textures = HashMap::new();

        for (wall, path) in DEFAULT_TEXTURE_PATHS {
            if let Ok(texture) = Texture::from_file(path) {
                textures.insert(wall, texture);
            }
        }

        Self { textures, fallback }
    }

    pub fn get(&self, wall: char) -> &Texture {
        self.textures.get(&wall).unwrap_or(&self.fallback)
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

#[cfg(test)]
mod tests {
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

        TextureManager::new(textures, Texture::fallback())
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
    fn texture_manager_maps_known_wall_characters() {
        let manager = texture_manager_for_tests();

        assert_eq!(manager.get('#').get_pixel(0, 0), 0x111111);
        assert_eq!(manager.get('+').get_pixel(0, 0), 0x222222);
        assert_eq!(manager.get('%').get_pixel(0, 0), 0x333333);
        assert_eq!(manager.get('@').get_pixel(0, 0), 0x444444);
    }

    #[test]
    fn texture_manager_returns_fallback_for_unknown_wall_character() {
        let manager = texture_manager_for_tests();

        assert!(std::ptr::eq(manager.get('&'), &manager.fallback));
        assert_eq!(manager.get('&').get_pixel(0, 0), FALLBACK_MAGENTA);
    }

    #[test]
    fn texture_manager_get_returns_references_without_cloning() {
        let manager = texture_manager_for_tests();
        let stored_texture = manager.textures.get(&'#').unwrap();

        assert!(std::ptr::eq(manager.get('#'), stored_texture));
        assert!(std::ptr::eq(manager.get('#'), manager.get('#')));
    }
}
