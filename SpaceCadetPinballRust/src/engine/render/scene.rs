use crate::assets::DatFile;
use crate::engine::geom::RectI;

use super::sprite::SpriteRecord;

#[derive(Debug, Clone)]
pub struct RenderScene {
    pub asset_revision: u64,
    pub sprites: Vec<SpriteRecord>,
}

impl RenderScene {
    pub fn from_dat_file(dat_file: &DatFile, resolution: usize, asset_revision: u64) -> Self {
        let mut sprites = Vec::new();

        if let Some(group_index) = dat_file.background_group_index() {
            if let Some(group) = dat_file.group(group_index)
                && let Some(bitmap) = group.get_bitmap(resolution)
            {
                sprites.push(SpriteRecord::from_bitmap(
                    group_index,
                    bitmap,
                    group.get_zmap(resolution),
                ));
            }
        } else {
            for (group_index, group) in dat_file.groups.iter().enumerate() {
                if let Some(bitmap) = group.get_bitmap(resolution) {
                    sprites.push(SpriteRecord::from_bitmap(
                        group_index,
                        bitmap,
                        group.get_zmap(resolution),
                    ));
                }
            }
        }

        if let Some(table_group) = dat_file.table_group_index()
            && let Some(group) = dat_file.group(table_group)
            && let Some(bitmap) = group.get_bitmap(resolution)
        {
            let depth_hint = group
                .get_zmap(resolution)
                .map(|map| i32::from(map.average_sample()))
                .unwrap_or(i32::from(bitmap.y_position));
            sprites.push(SpriteRecord::at_dest(
                table_group,
                bitmap.resolution,
                RectI::new(0, 0, bitmap.width as u32, bitmap.height as u32),
                depth_hint,
            ));
        }

        sprites.sort_by_key(|sprite| (sprite.depth_hint, sprite.dest.y, sprite.key.group_index));

        Self {
            asset_revision,
            sprites,
        }
    }
}
