use crate::engine::bitmap::{Bitmap8, ZMap};
use crate::engine::geom::RectI;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SpriteKey {
    pub group_index: usize,
    pub resolution: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct SpriteRecord {
    pub key: SpriteKey,
    pub dest: RectI,
    pub depth_hint: i32,
}

impl SpriteRecord {
    pub fn from_bitmap(group_index: usize, bitmap: &Bitmap8, zmap: Option<&ZMap>) -> Self {
        let depth_hint = zmap
            .map(|map| i32::from(map.average_sample()))
            .unwrap_or_else(|| i32::from(bitmap.y_position));

        Self {
            key: SpriteKey {
                group_index,
                resolution: bitmap.resolution,
            },
            dest: RectI::new(
                i32::from(bitmap.x_position),
                i32::from(bitmap.y_position),
                bitmap.width as u32,
                bitmap.height as u32,
            ),
            depth_hint,
        }
    }

    pub fn at_dest(group_index: usize, resolution: usize, dest: RectI, depth_hint: i32) -> Self {
        Self {
            key: SpriteKey {
                group_index,
                resolution,
            },
            dest,
            depth_hint,
        }
    }
}
