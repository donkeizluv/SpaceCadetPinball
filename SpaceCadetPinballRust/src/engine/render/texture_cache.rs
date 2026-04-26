use std::collections::HashMap;

use sdl2::pixels::PixelFormatEnum;
use sdl2::render::{BlendMode, Canvas, Texture, TextureCreator};
use sdl2::video::{Window, WindowContext};

use crate::assets::DatFile;

use super::sprite::{SpriteKey, SpriteRecord};

#[derive(Debug, Clone, Copy)]
struct PaletteColor {
    red: u8,
    green: u8,
    blue: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct TextureKey {
    asset_revision: u64,
    sprite: SpriteKey,
}

pub struct TextureCache {
    textures: HashMap<TextureKey, Texture>,
}

impl TextureCache {
    pub fn new() -> Self {
        Self {
            textures: HashMap::new(),
        }
    }

    pub fn prune(&mut self, asset_revision: u64) {
        self.textures
            .retain(|key, _| key.asset_revision == asset_revision);
    }

    pub fn draw_sprite(
        &mut self,
        canvas: &mut Canvas<Window>,
        texture_creator: &TextureCreator<WindowContext>,
        dat_file: &DatFile,
        sprite: &SpriteRecord,
        asset_revision: u64,
    ) -> Result<(), String> {
        let key = TextureKey {
            asset_revision,
            sprite: sprite.key,
        };

        if !self.textures.contains_key(&key) {
            let texture = create_texture(texture_creator, dat_file, sprite)?;
            self.textures.insert(key, texture);
        }

        let texture = self
            .textures
            .get(&key)
            .ok_or_else(|| "texture cache entry missing after insert".to_string())?;
        canvas
            .copy(texture, None, Some(sprite.dest.to_sdl_rect()))
            .map_err(|error| error.to_string())
    }
}

fn create_texture(
    texture_creator: &TextureCreator<WindowContext>,
    dat_file: &DatFile,
    sprite: &SpriteRecord,
) -> Result<Texture, String> {
    let bitmap = dat_file
        .get_bitmap(sprite.key.group_index, sprite.key.resolution)
        .ok_or_else(|| {
            format!(
                "missing bitmap for group {} at resolution {}",
                sprite.key.group_index, sprite.key.resolution
            )
        })?;

    let mut texture = texture_creator
        .create_texture_streaming(
            PixelFormatEnum::RGBA8888,
            bitmap.width as u32,
            bitmap.height as u32,
        )
        .map_err(|error| error.to_string())?;
    texture.set_blend_mode(BlendMode::Blend);
    let palette = resolve_palette(dat_file)?;
    texture
        .with_lock(None, |buffer: &mut [u8], pitch: usize| {
            for y in 0..bitmap.height {
                let src_y = bitmap.height - 1 - y;
                let src_row = src_y * bitmap.indexed_stride;
                let dst_row = y * pitch;
                for x in 0..bitmap.width {
                    let sample = bitmap.indexed_pixels[src_row + x];
                    let dst = dst_row + x * 4;
                    let color = palette[usize::from(sample)];

                    buffer[dst] = color.red;
                    buffer[dst + 1] = color.green;
                    buffer[dst + 2] = color.blue;
                    buffer[dst + 3] = if sample == 0 { 0 } else { 255 };
                }
            }
        })
        .map_err(|error| error.to_string())?;

    Ok(texture)
}

fn resolve_palette(dat_file: &DatFile) -> Result<[PaletteColor; 256], String> {
    let bytes = dat_file
        .background_palette_bytes()
        .ok_or_else(|| "missing background palette record".to_string())?;

    if bytes.len() != 256 * 4 {
        return Err(format!(
            "background palette size mismatch, expected 1024 bytes, got {}",
            bytes.len()
        ));
    }

    let mut palette = [PaletteColor {
        red: 0,
        green: 0,
        blue: 0,
    }; 256];

    for (index, chunk) in bytes.chunks_exact(4).enumerate() {
        let color = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        palette[index] = PaletteColor {
            red: ((color >> 16) & 0xFF) as u8,
            green: ((color >> 8) & 0xFF) as u8,
            blue: (color & 0xFF) as u8,
        };
    }

    Ok(palette)
}
