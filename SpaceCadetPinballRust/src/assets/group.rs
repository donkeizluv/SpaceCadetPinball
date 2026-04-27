use super::{Bitmap8, BitmapType, DatFile, EntryData, EntryPayload, FieldType, GroupData, ZMap};

#[derive(Debug, Clone, Copy)]
pub struct HudWidgetLayout {
    pub digits: usize,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct TextBoxLayout {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct SequenceFrame {
    pub group_index: usize,
    pub frame_index: usize,
    pub frame_count: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct NamedBitmapFrame {
    pub group_index: usize,
    pub bitmap_resolution: usize,
}

#[derive(Debug, Clone, Copy)]
pub struct MessageFontGlyph {
    pub group_index: usize,
    pub bitmap_resolution: usize,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct MessageFont {
    pub gap_width: i32,
    pub line_height: i32,
    glyphs: Vec<Option<MessageFontGlyph>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VisualCollisionMetadata {
    pub collision_group: u32,
    pub smoothness: f32,
    pub elasticity: f32,
    pub threshold: f32,
    pub boost: f32,
    pub soft_hit_sound_id: i32,
    pub hard_hit_sound_id: i32,
    pub wall_float_count: usize,
}

impl MessageFont {
    pub fn glyph(&self, character: u8) -> Option<MessageFontGlyph> {
        self.glyphs.get(character as usize).copied().flatten()
    }
}

impl GroupData {
    pub(crate) fn new(group_id: usize) -> Self {
        Self {
            group_id,
            group_name: None,
            entries: Vec::new(),
            bitmaps: [None, None, None],
            zmaps: [None, None, None],
            needs_sort: false,
        }
    }

    pub(crate) fn add_entry(&mut self, entry: EntryData) -> Result<(), String> {
        let EntryData {
            entry_type,
            field_size,
            payload,
        } = entry;

        match payload {
            EntryPayload::Text(text) if entry_type == FieldType::GroupName => {
                self.group_name = Some(text.clone());
                self.entries.push(EntryData {
                    entry_type,
                    field_size,
                    payload: EntryPayload::Text(text),
                });
            }
            EntryPayload::Bitmap8(bitmap) => {
                if bitmap.bitmap_type == BitmapType::Spliced {
                    let (split_bmp, split_zmap) = split_spliced_bitmap(&bitmap)?;
                    self.needs_sort = true;
                    self.set_bitmap(split_bmp.clone())?;
                    self.set_zmap(split_zmap.clone())?;
                    self.entries.push(EntryData {
                        entry_type: FieldType::Bitmap8bit,
                        field_size: 0,
                        payload: EntryPayload::Bitmap8Ref(split_bmp.resolution),
                    });
                    self.entries.push(EntryData {
                        entry_type: FieldType::Bitmap16bit,
                        field_size: 0,
                        payload: EntryPayload::Bitmap16Ref(split_zmap.resolution),
                    });
                } else {
                    self.set_bitmap(bitmap.clone())?;
                    self.entries.push(EntryData {
                        entry_type,
                        field_size,
                        payload: EntryPayload::Bitmap8Ref(bitmap.resolution),
                    });
                }
            }
            EntryPayload::Bitmap16(zmap) => {
                self.set_zmap(zmap.clone())?;
                self.entries.push(EntryData {
                    entry_type,
                    field_size,
                    payload: EntryPayload::Bitmap16Ref(zmap.resolution),
                });
            }
            other => {
                self.entries.push(EntryData {
                    entry_type,
                    field_size,
                    payload: other,
                });
            }
        }

        Ok(())
    }

    fn set_bitmap(&mut self, bitmap: Bitmap8) -> Result<(), String> {
        if bitmap.resolution >= self.bitmaps.len() {
            return Err(format!(
                "group {}: bitmap resolution {} out of bounds",
                self.group_id, bitmap.resolution
            ));
        }

        if self.bitmaps[bitmap.resolution].is_some() {
            return Err(format!(
                "group {}: bitmap override at resolution {}",
                self.group_id, bitmap.resolution
            ));
        }

        if let Some(zmap) = &self.zmaps[bitmap.resolution]
            && (bitmap.width != zmap.width || bitmap.height != zmap.height)
        {
            return Err(format!(
                "group {}: mismatched bitmap/zMap dimensions",
                self.group_id
            ));
        }

        let resolution = bitmap.resolution;
        self.bitmaps[resolution] = Some(bitmap);
        Ok(())
    }

    fn set_zmap(&mut self, mut zmap: ZMap) -> Result<(), String> {
        flip_zmap_horizontally(&mut zmap);

        if zmap.resolution >= self.zmaps.len() {
            return Err(format!(
                "group {}: zMap resolution {} out of bounds",
                self.group_id, zmap.resolution
            ));
        }

        if self.zmaps[zmap.resolution].is_some() {
            return Err(format!(
                "group {}: zMap override at resolution {}",
                self.group_id, zmap.resolution
            ));
        }

        if let Some(bitmap) = &self.bitmaps[zmap.resolution]
            && (bitmap.width != zmap.width || bitmap.height != zmap.height)
        {
            return Err(format!(
                "group {}: mismatched bitmap/zMap dimensions",
                self.group_id
            ));
        }

        let resolution = zmap.resolution;
        self.zmaps[resolution] = Some(zmap);
        Ok(())
    }

    pub(crate) fn finalize_group(&mut self) {
        if self.needs_sort {
            self.needs_sort = false;
            self.entries.sort_by_key(|entry| entry.entry_type as u8);
            self.entries.shrink_to_fit();
        }
    }
}

impl GroupData {
    pub fn get_bitmap(&self, resolution: usize) -> Option<&Bitmap8> {
        self.bitmaps.get(resolution).and_then(Option::as_ref)
    }

    pub fn get_zmap(&self, resolution: usize) -> Option<&ZMap> {
        self.zmaps.get(resolution).and_then(Option::as_ref)
    }

    pub fn find_entry(&self, entry_type: FieldType) -> Option<&EntryData> {
        self.entries
            .iter()
            .find(|entry| entry.entry_type == entry_type)
    }

    pub fn text_value(&self, entry_type: FieldType) -> Option<&str> {
        match &self.find_entry(entry_type)?.payload {
            EntryPayload::Text(text) => Some(text.as_str()),
            _ => None,
        }
    }

    pub fn raw_bytes(&self, entry_type: FieldType) -> Option<&[u8]> {
        match &self.find_entry(entry_type)?.payload {
            EntryPayload::RawBytes(bytes) => Some(bytes.as_slice()),
            _ => None,
        }
    }

    pub fn short_values(&self, entry_type: FieldType) -> Option<Vec<i16>> {
        let bytes = self.raw_bytes(entry_type)?;
        Some(
            bytes
                .chunks_exact(2)
                .map(|chunk| i16::from_le_bytes([chunk[0], chunk[1]]))
                .collect(),
        )
    }

    pub fn float_values(&self, entry_type: FieldType) -> Option<Vec<f32>> {
        let bytes = self.raw_bytes(entry_type)?;
        Some(
            bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect(),
        )
    }
}

impl DatFile {
    pub(crate) fn finalize(&mut self) {
        for group in &mut self.groups {
            group.finalize_group();
        }
    }

    pub fn group(&self, group_index: usize) -> Option<&GroupData> {
        self.groups.get(group_index)
    }

    pub fn group_label(&self, group_index: usize) -> Option<&str> {
        self.group(group_index)?.group_name.as_deref()
    }

    pub fn group_named(&self, target_group_name: &str) -> Option<&GroupData> {
        self.groups
            .iter()
            .rev()
            .find(|group| group.group_name.as_deref() == Some(target_group_name))
    }

    pub fn record_labeled(&self, target_group_name: &str) -> Option<usize> {
        self.groups
            .iter()
            .rposition(|group| group.group_name.as_deref() == Some(target_group_name))
    }

    pub fn get_bitmap(&self, group_index: usize, resolution: usize) -> Option<&Bitmap8> {
        self.group(group_index)?.get_bitmap(resolution)
    }

    pub fn get_zmap(&self, group_index: usize, resolution: usize) -> Option<&ZMap> {
        self.group(group_index)?.get_zmap(resolution)
    }

    pub fn background_group_index(&self) -> Option<usize> {
        self.record_labeled("background")
    }

    pub fn table_group_index(&self) -> Option<usize> {
        self.record_labeled("table")
    }

    pub fn table_bitmap_origin(&self, resolution: usize) -> Option<(i32, i32)> {
        let group_index = self.table_group_index()?;
        let bitmap = self.get_bitmap(group_index, resolution)?;
        Some((i32::from(bitmap.x_position), i32::from(bitmap.y_position)))
    }

    pub fn palette_bytes_for_group(&self, group_index: usize) -> Option<&[u8]> {
        self.group(group_index)?.raw_bytes(FieldType::Palette)
    }

    pub fn background_palette_bytes(&self) -> Option<&[u8]> {
        let group_index = self.background_group_index()?;
        self.palette_bytes_for_group(group_index)
    }

    pub fn named_bitmap_frame(
        &self,
        target_group_name: &str,
        resolution: usize,
    ) -> Option<NamedBitmapFrame> {
        let group_index = self.record_labeled(target_group_name)?;
        let bitmap = self.get_bitmap(group_index, resolution)?;

        Some(NamedBitmapFrame {
            group_index,
            bitmap_resolution: bitmap.resolution,
        })
    }

    pub fn bitmap_sequence_indices(
        &self,
        target_group_name: &str,
        resolution: usize,
    ) -> Option<Vec<usize>> {
        let start_index = self.record_labeled(target_group_name)?;
        let mut indices = Vec::new();

        for (group_index, group) in self.groups.iter().enumerate().skip(start_index) {
            if group_index > start_index && group.group_name.is_some() {
                break;
            }

            if group.get_bitmap(resolution).is_some() {
                indices.push(group_index);
            }
        }

        if indices.is_empty() {
            None
        } else {
            Some(indices)
        }
    }

    pub fn sequence_frame(
        &self,
        target_group_name: &str,
        resolution: usize,
        frame_fraction: f32,
    ) -> Option<SequenceFrame> {
        let sequence = self.bitmap_sequence_indices(target_group_name, resolution)?;
        let frame_count = sequence.len();
        let frame_index = if frame_count <= 1 {
            0
        } else {
            (frame_fraction.clamp(0.0, 1.0) * (frame_count - 1) as f32).round() as usize
        };

        Some(SequenceFrame {
            group_index: sequence[frame_index.min(frame_count - 1)],
            frame_index,
            frame_count,
        })
    }

    pub fn number_widget_digit_groups(
        &self,
        font_group_name: &str,
        resolution: usize,
        value: u64,
        digit_limit: usize,
    ) -> Option<Vec<usize>> {
        let font_sequence = self.bitmap_sequence_indices(font_group_name, resolution)?;
        let mut digits = value.to_string();
        if digits.len() > digit_limit {
            let start = digits.len().saturating_sub(digit_limit);
            digits = digits[start..].to_string();
        }

        let mut groups = Vec::with_capacity(digits.len().max(1));
        for digit_char in digits.chars() {
            let digit = digit_char.to_digit(10)? as usize;
            let &group_index = font_sequence.get(digit)?;
            groups.push(group_index);
        }

        Some(groups)
    }

    pub fn hud_widget_layout(&self, target_group_name: &str) -> Option<HudWidgetLayout> {
        let group = self.group_named(target_group_name)?;
        let bytes = group.raw_bytes(FieldType::ShortArray)?;
        if bytes.len() < 10 {
            return None;
        }

        let mut values = [0i16; 5];
        for (index, chunk) in bytes.chunks_exact(2).take(5).enumerate() {
            values[index] = i16::from_le_bytes([chunk[0], chunk[1]]);
        }

        Some(HudWidgetLayout {
            digits: values[0].max(1) as usize,
            x: i32::from(values[1]),
            y: i32::from(values[2]),
            width: values[3].max(1) as u32,
            height: values[4].max(1) as u32,
        })
    }

    pub fn text_box_layout(&self, target_group_name: &str) -> Option<TextBoxLayout> {
        let group = self.group_named(target_group_name)?;
        let bytes = group.raw_bytes(FieldType::ShortArray)?;
        if bytes.len() < 10 {
            return None;
        }

        let mut values = [0i16; 5];
        for (index, chunk) in bytes.chunks_exact(2).take(5).enumerate() {
            values[index] = i16::from_le_bytes([chunk[0], chunk[1]]);
        }

        Some(TextBoxLayout {
            x: i32::from(values[1]),
            y: i32::from(values[2]),
            width: values[3].max(1) as u32,
            height: values[4].max(1) as u32,
        })
    }

    pub fn message_font(&self, target_group_name: &str, resolution: usize) -> Option<MessageFont> {
        let start_index = self.record_labeled(target_group_name)?;
        let gap_width = self
            .group(start_index)
            .and_then(|group| group.raw_bytes(FieldType::ShortArray))
            .and_then(|bytes| {
                let offset = resolution.checked_mul(2)?;
                let chunk = bytes.get(offset..offset + 2)?;
                Some(i32::from(i16::from_le_bytes([chunk[0], chunk[1]])))
            })
            .unwrap_or(0);

        let mut glyphs = vec![None; 128];
        let mut line_height = 0i32;

        for (character, group_index) in (32u8..128u8).zip(start_index..) {
            let Some(bitmap) = self.get_bitmap(group_index, resolution) else {
                break;
            };

            glyphs[character as usize] = Some(MessageFontGlyph {
                group_index,
                bitmap_resolution: bitmap.resolution,
                width: bitmap.width as u32,
                height: bitmap.height as u32,
            });
            line_height = line_height.max(bitmap.height as i32);
        }

        if line_height == 0 {
            None
        } else {
            Some(MessageFont {
                gap_width,
                line_height,
                glyphs,
            })
        }
    }

    pub fn visual_collision_metadata(
        &self,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<VisualCollisionMetadata> {
        let state_index = self.visual_state_group_index(group_index, group_index_offset)?;
        let mut metadata = VisualCollisionMetadata::default();

        if let Some(short_values) = self
            .group(state_index)
            .and_then(|group| group.short_values(FieldType::ShortArray))
        {
            let mut index = 0;
            while index + 1 < short_values.len() {
                match short_values[index] {
                    300 => {
                        let material_group = short_values[index + 1].max(0) as usize;
                        self.apply_material_metadata(material_group, &mut metadata);
                    }
                    304 => metadata.soft_hit_sound_id = i32::from(short_values[index + 1]),
                    400 => {
                        let kicker_group = short_values[index + 1].max(0) as usize;
                        self.apply_kicker_metadata(kicker_group, &mut metadata);
                    }
                    406 => metadata.hard_hit_sound_id = i32::from(short_values[index + 1]),
                    602 => {
                        let shift = short_values[index + 1].max(0) as u32;
                        metadata.collision_group |= 1_u32.checked_shl(shift).unwrap_or(0);
                    }
                    1500 => {
                        index += 7;
                        continue;
                    }
                    _ => {}
                }
                index += 2;
            }
        }

        if metadata.collision_group == 0 {
            metadata.collision_group = 1;
        }

        if let Some(float_values) = self
            .group(state_index)
            .and_then(|group| group.float_values(FieldType::FloatArray))
            && float_values.first().copied() == Some(600.0)
            && float_values.len() >= 2
        {
            let raw_wall_type = float_values[1].floor() as isize - 1;
            metadata.wall_float_count = match raw_wall_type {
                0 => 1,
                1 => 2,
                value if value > 1 => value as usize,
                _ => 0,
            };
        }

        Some(metadata)
    }

    fn visual_state_group_index(
        &self,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<usize> {
        let state_count = self.query_visual_states(group_index)?;
        if group_index_offset > state_count {
            return None;
        }
        if group_index_offset == 0 {
            return Some(group_index);
        }

        let state_index = group_index.checked_add(group_index_offset)?;
        let short_values = self
            .group(state_index)?
            .short_values(FieldType::ShortValue)?;
        match short_values.first().copied() {
            Some(201) => Some(state_index),
            _ => None,
        }
    }

    fn query_visual_states(&self, group_index: usize) -> Option<usize> {
        let short_values = self
            .group(group_index)?
            .short_values(FieldType::ShortValue)?;
        if short_values.first().copied()? != 200 {
            return None;
        }
        short_values
            .get(1)
            .copied()
            .map(|value| value.max(0) as usize)
    }

    fn apply_material_metadata(
        &self,
        material_group: usize,
        metadata: &mut VisualCollisionMetadata,
    ) {
        let Some(float_values) = self
            .group(material_group)
            .and_then(|group| group.float_values(FieldType::FloatArray))
        else {
            return;
        };

        for pair in float_values.chunks_exact(2) {
            match pair[0].floor() as i32 {
                301 => metadata.smoothness = pair[1],
                302 => metadata.elasticity = pair[1],
                304 => metadata.soft_hit_sound_id = pair[1].floor() as i32,
                _ => {}
            }
        }
    }

    fn apply_kicker_metadata(&self, kicker_group: usize, metadata: &mut VisualCollisionMetadata) {
        let Some(float_values) = self
            .group(kicker_group)
            .and_then(|group| group.float_values(FieldType::FloatArray))
        else {
            return;
        };

        let mut index = 0;
        while index + 1 < float_values.len() {
            match float_values[index].floor() as i32 {
                401 => metadata.threshold = float_values[index + 1],
                402 => metadata.boost = float_values[index + 1],
                404 => index += 4,
                406 => metadata.hard_hit_sound_id = float_values[index + 1].floor() as i32,
                _ => {}
            }
            index += 2;
        }
    }
}

impl Default for VisualCollisionMetadata {
    fn default() -> Self {
        Self {
            collision_group: 0,
            smoothness: 0.95,
            elasticity: 0.6,
            threshold: 8.9999999e10,
            boost: 0.0,
            soft_hit_sound_id: 0,
            hard_hit_sound_id: 0,
            wall_float_count: 0,
        }
    }
}

fn split_spliced_bitmap(bitmap: &Bitmap8) -> Result<(Bitmap8, ZMap), String> {
    let width = bitmap.width;
    let height = bitmap.height;
    if width == 0 || height == 0 {
        return Err("spliced bitmap has empty dimensions".to_string());
    }

    let expected_pixels = width
        .checked_mul(height)
        .and_then(|count| count.checked_mul(3))
        .ok_or_else(|| "spliced bitmap size overflow".to_string())?;
    if bitmap.indexed_pixels.len() != expected_pixels {
        return Err(format!(
            "spliced bitmap payload size mismatch, expected {}, got {}",
            expected_pixels,
            bitmap.indexed_pixels.len()
        ));
    }

    let mut indexed_pixels = Vec::with_capacity(width * height);
    let mut zmap_samples = Vec::with_capacity(width * height);

    for pixel in bitmap.indexed_pixels.chunks_exact(3) {
        indexed_pixels.push(pixel[0]);
        zmap_samples.push(u16::from_le_bytes([pixel[1], pixel[2]]));
    }

    Ok((
        Bitmap8 {
            width,
            height,
            stride: width,
            indexed_stride: width,
            x_position: bitmap.x_position,
            y_position: bitmap.y_position,
            resolution: bitmap.resolution,
            bitmap_type: BitmapType::RawBitmap,
            indexed_pixels,
        },
        ZMap {
            width,
            height,
            stride: width,
            resolution: bitmap.resolution,
            samples: zmap_samples,
        },
    ))
}

fn flip_zmap_horizontally(zmap: &mut ZMap) {
    if zmap.width == 0 || zmap.height == 0 || zmap.stride == 0 {
        return;
    }

    for row in zmap.samples.chunks_exact_mut(zmap.stride).take(zmap.height) {
        row[..zmap.width].reverse();
    }
}
