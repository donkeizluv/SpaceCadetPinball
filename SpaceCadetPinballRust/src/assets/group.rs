use crate::engine::geom::RectI;

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
pub struct CameraProjection {
    pub matrix: [[f32; 4]; 3],
    pub distance: f32,
    pub z_min: f32,
    pub z_scaler: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TableView {
    pub resolution: usize,
    pub table_origin: (i32, i32),
    pub projection_center: Option<(f32, f32)>,
    pub projection: Option<CameraProjection>,
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualCollisionEdge {
    Line { start: (f32, f32), end: (f32, f32) },
    Circle { center: (f32, f32), radius: f32 },
}

impl MessageFont {
    pub fn glyph(&self, character: u8) -> Option<MessageFontGlyph> {
        self.glyphs.get(character as usize).copied().flatten()
    }
}

impl CameraProjection {
    fn matrix_vector_multiply(self, world_x: f32, world_y: f32, world_z: f32) -> (f32, f32, f32) {
        (
            world_x * self.matrix[0][0]
                + world_y * self.matrix[0][1]
                + world_z * self.matrix[0][2]
                + self.matrix[0][3],
            world_x * self.matrix[1][0]
                + world_y * self.matrix[1][1]
                + world_z * self.matrix[1][2]
                + self.matrix[1][3],
            world_x * self.matrix[2][0]
                + world_y * self.matrix[2][1]
                + world_z * self.matrix[2][2]
                + self.matrix[2][3],
        )
    }

    pub fn z_distance(self, world_x: f32, world_y: f32, world_z: f32) -> f32 {
        let (x, y, z) = self.matrix_vector_multiply(world_x, world_y, world_z);
        (x * x + y * y + z * z).sqrt()
    }

    pub fn project_to_2d(self, world_x: f32, world_y: f32, world_z: f32, center_x: f32, center_y: f32) -> Option<(f32, f32)> {
        let (projected_x, projected_y, projected_z) =
            self.matrix_vector_multiply(world_x, world_y, world_z);

        if projected_z.abs() <= f32::EPSILON {
            return None;
        }

        Some((
            projected_x * self.distance / projected_z + center_x,
            projected_y * self.distance / projected_z + center_y,
        ))
    }

    pub fn reverse_project_to_world_plane(
        self,
        projected_x: f32,
        projected_y: f32,
        center_x: f32,
        center_y: f32,
        world_z: f32,
    ) -> Option<(f32, f32)> {
        // Source-backed from proj::ReverseXForm in the decompiled C++ port.
        // The original game uses one fixed perspective matrix family for tables,
        // and solves x/y on a caller-chosen z plane (most often z = 0).
        if self.distance.abs() <= f32::EPSILON {
            return None;
        }

        let a = self.matrix[1][1];
        let b = self.matrix[1][2];
        let f = self.matrix[1][3];
        let g = self.matrix[2][3];
        let x2 = (projected_x - center_x) / self.distance;
        let y2 = (projected_y - center_y) / self.distance;
        let denominator = a + b * y2;
        if denominator.abs() <= f32::EPSILON {
            return None;
        }

        let world_y = (y2 * (a * world_z + g) - b * world_z - f) / denominator;
        let world_x = x2 * (a * world_z - b * world_y + g);
        Some((world_x, world_y))
    }
}

impl TableView {
    pub fn bitmap_rect_to_table_local(self, mut rect: RectI) -> RectI {
        rect.x -= self.table_origin.0;
        rect.y -= self.table_origin.1;
        rect
    }

    pub fn project_world_point_to_table_local(
        self,
        world_x: f32,
        world_y: f32,
        world_z: f32,
    ) -> Option<(f32, f32)> {
        let projection = self.projection?;
        let (center_x, center_y) = self.projection_center?;
        let (x, y) = projection.project_to_2d(world_x, world_y, world_z, center_x, center_y)?;
        Some((x - self.table_origin.0 as f32, y - self.table_origin.1 as f32))
    }

    pub fn table_local_point_to_world_plane(
        self,
        local_x: f32,
        local_y: f32,
        world_z: f32,
    ) -> Option<(f32, f32)> {
        let projection = self.projection?;
        let (center_x, center_y) = self.projection_center?;
        projection.reverse_project_to_world_plane(
            local_x + self.table_origin.0 as f32,
            local_y + self.table_origin.1 as f32,
            center_x,
            center_y,
            world_z,
        )
    }

    pub fn project_world_centered_rect_to_table_local(
        self,
        world_x: f32,
        world_y: f32,
        world_z: f32,
        width: u32,
        height: u32,
    ) -> Option<RectI> {
        let (center_x, center_y) =
            self.project_world_point_to_table_local(world_x, world_y, world_z)?;
        Some(RectI::new(
            (center_x - width as f32 * 0.5).round() as i32,
            (center_y - height as f32 * 0.5).round() as i32,
            width.max(1),
            height.max(1),
        ))
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

    pub fn table_view(&self, resolution: usize) -> Option<TableView> {
        Some(TableView {
            resolution,
            table_origin: self.table_bitmap_origin(resolution)?,
            projection_center: self.table_projection_center(resolution),
            projection: self.camera_projection(resolution),
        })
    }

    pub fn camera_info_group_index(&self, resolution: usize) -> Option<usize> {
        self.record_labeled("camera_info")?.checked_add(resolution)
    }

    pub fn camera_projection(&self, resolution: usize) -> Option<CameraProjection> {
        let values = self
            .group(self.camera_info_group_index(resolution)?)?
            .float_values(FieldType::FloatArray)?;
        if values.len() < 15 {
            return None;
        }

        Some(CameraProjection {
            matrix: [
                [values[0], values[1], values[2], values[3]],
                [values[4], values[5], values[6], values[7]],
                [values[8], values[9], values[10], values[11]],
            ],
            distance: values[12],
            z_min: values[13],
            z_scaler: values[14],
        })
    }

    pub fn table_projection_center(&self, resolution: usize) -> Option<(f32, f32)> {
        let group_index = self.table_group_index()?;
        let values = self.float_attribute(group_index, 0, 700_i16.checked_add(resolution as i16)?)?;
        if values.len() < 2 {
            return None;
        }
        Some((values[0], values[1]))
    }

    pub fn table_collision_outline_points(&self) -> Option<Vec<(f32, f32)>> {
        let group_index = self.table_group_index()?;
        self.visual_primary_points(group_index, 0)
    }

    pub fn table_local_collision_outline_points(&self, resolution: usize) -> Option<Vec<(f32, f32)>> {
        let points = self.table_collision_outline_points()?;
        points
            .into_iter()
            .map(|(x, y)| self.project_world_point_to_table_local(resolution, x, y, 0.0))
            .collect()
    }

    pub fn bitmap_rect(&self, group_index: usize, resolution: usize) -> Option<RectI> {
        let bitmap = self.get_bitmap(group_index, resolution)?;
        Some(RectI::new(
            i32::from(bitmap.x_position),
            i32::from(bitmap.y_position),
            bitmap.width as u32,
            bitmap.height as u32,
        ))
    }

    pub fn table_local_bitmap_rect(&self, group_index: usize, resolution: usize) -> Option<RectI> {
        Some(
            self.table_view(resolution)?
                .bitmap_rect_to_table_local(self.bitmap_rect(group_index, resolution)?),
        )
    }

    pub fn project_world_point_to_table_local(
        &self,
        resolution: usize,
        world_x: f32,
        world_y: f32,
        world_z: f32,
    ) -> Option<(f32, f32)> {
        self.table_view(resolution)?
            .project_world_point_to_table_local(world_x, world_y, world_z)
    }

    pub fn table_local_point_to_world_plane(
        &self,
        resolution: usize,
        local_x: f32,
        local_y: f32,
        world_z: f32,
    ) -> Option<(f32, f32)> {
        self.table_view(resolution)?
            .table_local_point_to_world_plane(local_x, local_y, world_z)
    }

    pub fn point_attribute(
        &self,
        group_index: usize,
        group_index_offset: usize,
        first_value: i16,
    ) -> Option<(f32, f32)> {
        let values = self.float_attribute(group_index, group_index_offset, first_value)?;
        Some((*values.first()?, *values.get(1)?))
    }

    pub fn point3_attribute(
        &self,
        group_index: usize,
        group_index_offset: usize,
        first_value: i16,
    ) -> Option<(f32, f32, f32)> {
        let values = self.float_attribute(group_index, group_index_offset, first_value)?;
        Some((*values.first()?, *values.get(1)?, *values.get(2)?))
    }

    pub fn feed_position_world_point(
        &self,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<(f32, f32)> {
        self.point_attribute(group_index, group_index_offset, 601)
    }

    pub fn feed_position_table_local(
        &self,
        resolution: usize,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<(f32, f32)> {
        let (world_x, world_y) = self.feed_position_world_point(group_index, group_index_offset)?;
        // Source-backed from TPlunger/TSink reading attribute 601 as a 2D world point,
        // then rendering it through proj::xform_to_2d(vector2), which is the z = 0 plane.
        self.project_world_point_to_table_local(resolution, world_x, world_y, 0.0)
    }

    pub fn project_world_centered_rect_to_table_local(
        &self,
        resolution: usize,
        world_x: f32,
        world_y: f32,
        world_z: f32,
        width: u32,
        height: u32,
    ) -> Option<RectI> {
        self.table_view(resolution)?
            .project_world_centered_rect_to_table_local(world_x, world_y, world_z, width, height)
    }

    pub fn depth_sorted_sequence_frame(
        &self,
        target_group_name: &str,
        resolution: usize,
        world_x: f32,
        world_y: f32,
        world_z: f32,
    ) -> Option<SequenceFrame> {
        let projection = self.camera_projection(resolution)?;
        let start_index = self.record_labeled(target_group_name)?;
        let sequence = self.bitmap_sequence_indices(target_group_name, resolution)?;
        let frame_count = sequence.len();
        if frame_count == 0 {
            return None;
        }

        let target_depth = projection.z_distance(world_x, world_y, world_z);
        let mut frame_index = frame_count - 1;

        for candidate_index in 0..frame_count.saturating_sub(1) {
            let (sample_x, sample_y, sample_z) =
                self.point3_attribute(start_index, candidate_index, 501)?;
            let sample_depth = projection.z_distance(sample_x, sample_y, sample_z);
            if sample_depth <= target_depth {
                frame_index = candidate_index;
                break;
            }
        }

        Some(SequenceFrame {
            group_index: sequence[frame_index],
            frame_index,
            frame_count,
        })
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

    pub fn named_bitmap_rect(&self, target_group_name: &str, resolution: usize) -> Option<RectI> {
        let frame = self.named_bitmap_frame(target_group_name, resolution)?;
        self.bitmap_rect(frame.group_index, frame.bitmap_resolution)
    }

    pub fn table_local_named_bitmap_rect(
        &self,
        target_group_name: &str,
        resolution: usize,
    ) -> Option<RectI> {
        let frame = self.named_bitmap_frame(target_group_name, resolution)?;
        self.table_local_bitmap_rect(frame.group_index, frame.bitmap_resolution)
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

    pub fn sequence_frame_rect(
        &self,
        target_group_name: &str,
        resolution: usize,
        frame_fraction: f32,
    ) -> Option<RectI> {
        let frame = self.sequence_frame(target_group_name, resolution, frame_fraction)?;
        self.bitmap_rect(frame.group_index, resolution)
    }

    pub fn table_local_sequence_frame_rect(
        &self,
        target_group_name: &str,
        resolution: usize,
        frame_fraction: f32,
    ) -> Option<RectI> {
        let frame = self.sequence_frame(target_group_name, resolution, frame_fraction)?;
        self.table_local_bitmap_rect(frame.group_index, resolution)
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

    pub fn visual_collision_edges(
        &self,
        group_index: usize,
        group_index_offset: usize,
        first_value: i16,
        offset: f32,
    ) -> Option<Vec<VisualCollisionEdge>> {
        let float_values = self.float_attribute(group_index, group_index_offset, first_value)?;
        Self::decode_visual_collision_edges(&float_values, offset)
    }

    pub fn table_local_visual_collision_edges(
        &self,
        resolution: usize,
        group_index: usize,
        group_index_offset: usize,
        first_value: i16,
        offset: f32,
    ) -> Option<Vec<VisualCollisionEdge>> {
        let edges = self.visual_collision_edges(group_index, group_index_offset, first_value, offset)?;
        edges.into_iter()
            .map(|edge| match edge {
                VisualCollisionEdge::Line { start, end } => {
                    let start = self.project_world_point_to_table_local(resolution, start.0, start.1, 0.0)?;
                    let end = self.project_world_point_to_table_local(resolution, end.0, end.1, 0.0)?;
                    Some(VisualCollisionEdge::Line { start, end })
                }
                VisualCollisionEdge::Circle { center, radius } => {
                    let center_local =
                        self.project_world_point_to_table_local(resolution, center.0, center.1, 0.0)?;
                    let edge_local = self.project_world_point_to_table_local(
                        resolution,
                        center.0 + radius,
                        center.1,
                        0.0,
                    )?;
                    let projected_radius =
                        ((edge_local.0 - center_local.0).powi(2) + (edge_local.1 - center_local.1).powi(2))
                            .sqrt();
                    Some(VisualCollisionEdge::Circle {
                        center: center_local,
                        radius: if projected_radius == 0.0 { 0.001 } else { projected_radius },
                    })
                }
            })
            .collect()
    }

    pub fn visual_primary_points(
        &self,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<Vec<(f32, f32)>> {
        let state_index = self.visual_state_group_index(group_index, group_index_offset)?;
        let float_values = self
            .group(state_index)?
            .float_values(FieldType::FloatArray)?;
        Self::decode_visual_primary_points(&float_values)
    }

    pub fn table_local_visual_primary_points(
        &self,
        resolution: usize,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<Vec<(f32, f32)>> {
        let points = self.visual_primary_points(group_index, group_index_offset)?;
        points
            .into_iter()
            .map(|(x, y)| self.project_world_point_to_table_local(resolution, x, y, 0.0))
            .collect()
    }

    pub fn visual_circle_attribute_306(
        &self,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<VisualCollisionEdge> {
        let visual_values = self.float_attribute(group_index, group_index_offset, 600)?;
        if visual_values.first().copied()?.floor() as i32 - 1 != 0 {
            return None;
        }
        let center_x = *visual_values.get(1)?;
        let center_y = *visual_values.get(2)?;
        let radius_scale = self
            .float_attribute(group_index, group_index_offset, 306)?
            .first()
            .copied()
            .unwrap_or(0.0);
        let base_radius = *visual_values.get(3)?;
        let radius = radius_scale * base_radius;
        Some(VisualCollisionEdge::Circle {
            center: (center_x, center_y),
            radius: if radius == 0.0 { 0.001 } else { radius },
        })
    }

    pub fn table_local_visual_circle_attribute_306(
        &self,
        resolution: usize,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<VisualCollisionEdge> {
        let VisualCollisionEdge::Circle { center, radius } =
            self.visual_circle_attribute_306(group_index, group_index_offset)?
        else {
            return None;
        };
        let center_local =
            self.project_world_point_to_table_local(resolution, center.0, center.1, 0.0)?;
        let edge_local =
            self.project_world_point_to_table_local(resolution, center.0 + radius, center.1, 0.0)?;
        let projected_radius =
            ((edge_local.0 - center_local.0).powi(2) + (edge_local.1 - center_local.1).powi(2)).sqrt();
        Some(VisualCollisionEdge::Circle {
            center: center_local,
            radius: if projected_radius == 0.0 { 0.001 } else { projected_radius },
        })
    }

    pub fn float_attribute(
        &self,
        group_index: usize,
        group_index_offset: usize,
        first_value: i16,
    ) -> Option<Vec<f32>> {
        let state_index = self.visual_state_group_index(group_index, group_index_offset)?;

        self.group(state_index)?
            .entries
            .iter()
            .filter(|entry| entry.entry_type == FieldType::FloatArray)
            .filter_map(|entry| match &entry.payload {
                EntryPayload::RawBytes(bytes) => Some(bytes.as_slice()),
                _ => None,
            })
            .filter_map(|bytes| {
                let values: Vec<f32> = bytes
                    .chunks_exact(4)
                    .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                    .collect();
                if values.first().map(|value| value.floor() as i16) == Some(first_value) {
                    Some(values.into_iter().skip(1).collect())
                } else {
                    None
                }
            })
            .next()
    }

    fn visual_state_group_index(
        &self,
        group_index: usize,
        group_index_offset: usize,
    ) -> Option<usize> {
        if group_index_offset == 0 {
            return Some(group_index);
        }

        let state_count = self.query_visual_states(group_index)?;
        if group_index_offset > state_count {
            return None;
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
        let group = self.group(group_index)?;
        let short_values = group.short_values(FieldType::ShortArray);

        if let Some(short_values) = short_values
            && short_values.first().copied() == Some(100)
        {
            return short_values.get(1).copied().map(|value| value.max(1) as usize);
        }

        let marker = group.short_values(FieldType::ShortValue)?;
        if marker.first().copied()? == 200 {
            Some(1)
        } else {
            None
        }
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

    fn decode_visual_collision_edges(
        values: &[f32],
        offset: f32,
    ) -> Option<Vec<VisualCollisionEdge>> {
        let wall_value = values.first().copied()?;
        let wall_type = wall_value.floor() as i32 - 1;

        match wall_type {
            0 => {
                if values.len() < 4 {
                    return None;
                }
                Some(vec![VisualCollisionEdge::Circle {
                    center: (values[1], values[2]),
                    radius: values[3] + offset,
                }])
            }
            1 => {
                if values.len() < 5 {
                    return None;
                }
                Some(vec![VisualCollisionEdge::Line {
                    start: (values[1], values[2]),
                    end: (values[3], values[4]),
                }])
            }
            segment_count if segment_count > 1 => {
                let point_count = segment_count as usize;
                let expected_len = point_count.checked_mul(2)?.checked_add(1)?;
                if values.len() < expected_len {
                    return None;
                }

                let points: Vec<(f32, f32)> = values[1..expected_len]
                    .chunks_exact(2)
                    .map(|pair| (pair[0], pair[1]))
                    .collect();

                let mut edges = Vec::with_capacity(point_count + usize::from(offset != 0.0));
                for index in 0..point_count {
                    let start = points[index];
                    let end = points[(index + 1) % point_count];
                    edges.push(VisualCollisionEdge::Line { start, end });
                }

                if offset != 0.0 {
                    for index in 0..point_count {
                        let previous = points[(index + point_count - 1) % point_count];
                        let current = points[index];
                        let next = points[(index + 1) % point_count];
                        let cross = (current.0 - previous.0) * (next.1 - current.1)
                            - (current.1 - previous.1) * (next.0 - current.0);
                        if (cross > 0.0 && offset > 0.0) || (cross < 0.0 && offset < 0.0) {
                            edges.push(VisualCollisionEdge::Circle {
                                center: current,
                                radius: offset * 1.001,
                            });
                        }
                    }
                }

                Some(edges)
            }
            _ => None,
        }
    }

    fn decode_visual_primary_points(values: &[f32]) -> Option<Vec<(f32, f32)>> {
        if values.first().copied() != Some(600.0) || values.len() < 2 {
            return None;
        }

        let wall_type = values[1].floor() as i32 - 1;
        let point_count = match wall_type {
            0 => return None,
            1 => 2,
            count if count > 1 => count as usize,
            _ => return None,
        };

        let expected_len = point_count.checked_mul(2)?.checked_add(2)?;
        if values.len() < expected_len {
            return None;
        }

        Some(
            values[2..expected_len]
                .chunks_exact(2)
                .map(|pair| (pair[0], pair[1]))
                .collect(),
        )
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

#[cfg(test)]
mod tests {
    use crate::assets::{Bitmap8, BitmapType, DatFile, EntryData, EntryPayload, FieldType, GroupData};

    use super::{CameraProjection, VisualCollisionEdge, VisualCollisionMetadata};

    fn group_with_float_attribute(group_name: &str, values: &[f32]) -> GroupData {
        let mut group = GroupData::new(0);
        group.group_name = Some(group_name.to_string());
        group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes(
                [200_i16, 0_i16]
                    .into_iter()
                    .flat_map(i16::to_le_bytes)
                    .collect(),
            ),
        });
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: (values.len() * 4) as i32,
            payload: EntryPayload::RawBytes(values.iter().copied().flat_map(f32::to_le_bytes).collect()),
        });
        group
    }

    fn bitmap_group(group_id: usize, group_name: &str, x: i16, y: i16, width: usize, height: usize) -> GroupData {
        let mut group = GroupData::new(group_id);
        group.group_name = Some(group_name.to_string());
        group.bitmaps[0] = Some(Bitmap8 {
            width,
            height,
            stride: width,
            indexed_stride: width,
            x_position: x,
            y_position: y,
            resolution: 0,
            bitmap_type: BitmapType::RawBitmap,
            indexed_pixels: vec![0; width * height],
        });
        group
    }

    fn raw_float_group(group_id: usize, group_name: &str, values: &[f32]) -> GroupData {
        let mut group = GroupData::new(group_id);
        group.group_name = Some(group_name.to_string());
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: (values.len() * 4) as i32,
            payload: EntryPayload::RawBytes(values.iter().copied().flat_map(f32::to_le_bytes).collect()),
        });
        group
    }

    #[test]
    fn visual_collision_edges_decode_line_arrays() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group_with_float_attribute("wall", &[600.0, 2.0, 10.0, 20.0, 30.0, 40.0])],
        };

        let edges = dat
            .visual_collision_edges(0, 0, 600, 0.0)
            .expect("line wall should decode");
        assert_eq!(
            edges,
            vec![VisualCollisionEdge::Line {
                start: (10.0, 20.0),
                end: (30.0, 40.0)
            }]
        );
    }

    #[test]
    fn table_local_bitmap_rect_subtracts_table_origin() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                bitmap_group(0, "table", 137, 2, 365, 470),
                bitmap_group(1, "plunger", 461, 383, 11, 42),
            ],
        };

        assert_eq!(
            dat.table_local_bitmap_rect(1, 0),
            Some(crate::engine::geom::RectI::new(324, 381, 11, 42))
        );
    }

    #[test]
    fn table_view_wraps_origin_and_projection_metadata() {
        let mut table_group = bitmap_group(1, "table", 137, 2, 365, 470);
        table_group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes([200_i16, 0_i16].into_iter().flat_map(i16::to_le_bytes).collect()),
        });
        table_group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [700.0_f32, 183.0, 238.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, -0.913545, 0.406737, 3.791398,
                        0.0, -0.406737, -0.913545, 24.675402,
                        -400.000702, 19.501307, 4303.969727,
                    ],
                ),
                table_group,
                bitmap_group(2, "plunger", 461, 383, 11, 42),
            ],
        };

        let table_view = dat.table_view(0).expect("table view");
        assert_eq!(table_view.table_origin, (137, 2));
        assert_eq!(table_view.projection_center, Some((183.0, 238.0)));
        assert_eq!(
            table_view.bitmap_rect_to_table_local(dat.bitmap_rect(2, 0).expect("bitmap rect")),
            crate::engine::geom::RectI::new(324, 381, 11, 42)
        );
        let projected = table_view
            .project_world_point_to_table_local(-7.020939, 10.084854, 0.0)
            .expect("projected point");
        assert!((projected.0 - 182.50).abs() < 0.1, "{projected:?}");
        assert!((projected.1 - 341.41).abs() < 0.1, "{projected:?}");
    }

    #[test]
    fn table_view_round_trips_projection_on_world_zero_plane() {
        let mut table_group = bitmap_group(1, "table", 137, 2, 365, 470);
        table_group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes([200_i16, 0_i16].into_iter().flat_map(i16::to_le_bytes).collect()),
        });
        table_group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [700.0_f32, 183.0, 238.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, -0.913545, 0.406737, 3.791398,
                        0.0, -0.406737, -0.913545, 24.675402,
                        -400.000702, 19.501307, 4303.969727,
                    ],
                ),
                table_group,
            ],
        };

        let table_view = dat.table_view(0).expect("table view");
        let world_point = (-7.020939_f32, 10.084854_f32, 0.0_f32);
        let local = table_view
            .project_world_point_to_table_local(world_point.0, world_point.1, world_point.2)
            .expect("projected point");
        let restored = table_view
            .table_local_point_to_world_plane(local.0, local.1, world_point.2)
            .expect("restored point");

        assert!((restored.0 - world_point.0).abs() < 0.1, "{restored:?}");
        assert!((restored.1 - world_point.1).abs() < 0.1, "{restored:?}");
    }

    #[test]
    fn table_view_projects_world_centered_rect_to_table_local() {
        let mut table_group = bitmap_group(1, "table", 137, 2, 365, 470);
        table_group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes([200_i16, 0_i16].into_iter().flat_map(i16::to_le_bytes).collect()),
        });
        table_group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [700.0_f32, 183.0, 238.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, -0.913545, 0.406737, 3.791398,
                        0.0, -0.406737, -0.913545, 24.675402,
                        -400.000702, 19.501307, 4303.969727,
                    ],
                ),
                table_group,
            ],
        };

        let rect = dat
            .project_world_centered_rect_to_table_local(0, -7.020939, 10.084854, 0.0, 12, 12)
            .expect("projected rect");
        assert_eq!(rect, crate::engine::geom::RectI::new(177, 335, 12, 12));
    }

    #[test]
    fn camera_projection_z_distance_matches_proj_cpp_magnitude_behavior() {
        let projection = CameraProjection {
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 2.0, 0.0, 0.0],
                [0.0, 0.0, 3.0, 0.0],
            ],
            distance: 1.0,
            z_min: 0.0,
            z_scaler: 1.0,
        };

        let distance = projection.z_distance(2.0, 3.0, 4.0);
        assert!((distance - 184.0_f32.sqrt()).abs() < 0.001, "{distance}");
    }

    #[test]
    fn table_local_named_and_sequence_rects_share_bitmap_conversion() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                bitmap_group(0, "table", 137, 2, 365, 470),
                bitmap_group(1, "plunger", 461, 383, 11, 42),
                bitmap_group(2, "plunger", 462, 384, 11, 43),
            ],
        };

        assert_eq!(
            dat.table_local_named_bitmap_rect("plunger", 0),
            Some(crate::engine::geom::RectI::new(462 - 137, 384 - 2, 11, 43))
        );
        assert_eq!(
            dat.table_local_sequence_frame_rect("plunger", 0, 0.0),
            Some(crate::engine::geom::RectI::new(325, 382, 11, 43))
        );
        assert_eq!(
            dat.table_local_sequence_frame_rect("plunger", 0, 1.0),
            Some(crate::engine::geom::RectI::new(325, 382, 11, 43))
        );
    }

    #[test]
    fn table_collision_outline_points_decode_from_table_visual_payload() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group_with_float_attribute(
                "table",
                &[600.0, 5.0, -10.0, -20.0, 30.0, -20.0, 30.0, 40.0, -10.0, 40.0],
            )],
        };

        assert_eq!(
            dat.table_collision_outline_points(),
            Some(vec![(-10.0, -20.0), (30.0, -20.0), (30.0, 40.0), (-10.0, 40.0)])
        );
    }

    #[test]
    fn feed_position_world_point_decodes_attribute_601_pair() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group_with_float_attribute("plunger", &[601.0, 525.0, 315.0])],
        };

        assert_eq!(dat.feed_position_world_point(0, 0), Some((525.0, 315.0)));
    }

    #[test]
    fn depth_sorted_sequence_frame_uses_visual_501_thresholds_like_tball_repaint() {
        let mut first_frame = bitmap_group(1, "ball", 0, 0, 12, 12);
        first_frame.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 16,
            payload: EntryPayload::RawBytes(
                [501.0_f32, 0.0, 0.0, 10.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        let mut second_frame = bitmap_group(2, "ball", 0, 0, 12, 12);
        second_frame.group_name = None;
        second_frame.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 16,
            payload: EntryPayload::RawBytes(
                [501.0_f32, 0.0, 0.0, 3.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });

        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, 1.0, 0.0, 0.0,
                        0.0, 0.0, 1.0, 0.0,
                        1.0, 0.0, 1.0,
                    ],
                ),
                first_frame,
                second_frame,
            ],
        };

        let far_frame = dat
            .depth_sorted_sequence_frame("ball", 0, 0.0, 0.0, 12.0)
            .expect("far frame");
        assert_eq!(far_frame.frame_index, 0);
        assert_eq!(far_frame.group_index, 1);

        let near_frame = dat
            .depth_sorted_sequence_frame("ball", 0, 0.0, 0.0, 5.0)
            .expect("near frame");
        assert_eq!(near_frame.frame_index, 1);
        assert_eq!(near_frame.group_index, 2);
    }

    #[test]
    fn project_world_point_to_table_local_uses_camera_info_and_table_center() {
        let mut table_group = bitmap_group(1, "table", 137, 2, 365, 470);
        table_group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes([200_i16, 0_i16].into_iter().flat_map(i16::to_le_bytes).collect()),
        });
        table_group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [700.0_f32, 183.0, 238.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });

        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, -0.913545, 0.406737, 3.791398,
                        0.0, -0.406737, -0.913545, 24.675402,
                        -400.000702, 19.501307, 4303.969727,
                    ],
                ),
                table_group,
            ],
        };

        let projected = dat
            .project_world_point_to_table_local(0, -7.020939, 10.084854, 0.0)
            .expect("projection should succeed");
        assert!((projected.0 - 182.50).abs() < 0.1, "{projected:?}");
        assert!((projected.1 - 341.41).abs() < 0.1, "{projected:?}");
    }

    #[test]
    fn feed_position_table_local_projects_world_plane_attribute_601() {
        let mut plunger = GroupData::new(2);
        plunger.group_name = Some("plunger".to_string());
        plunger.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [601.0_f32, -7.020939, 10.084854]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });
        let mut table_group = bitmap_group(1, "table", 137, 2, 365, 470);
        table_group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes([200_i16, 0_i16].into_iter().flat_map(i16::to_le_bytes).collect()),
        });
        table_group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [700.0_f32, 183.0, 238.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });

        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, -0.913545, 0.406737, 3.791398,
                        0.0, -0.406737, -0.913545, 24.675402,
                        -400.000702, 19.501307, 4303.969727,
                    ],
                ),
                table_group,
                plunger,
            ],
        };

        let projected = dat
            .feed_position_table_local(0, 2, 0)
            .expect("feed position should project");
        assert!((projected.0 - 182.50).abs() < 0.1, "{projected:?}");
        assert!((projected.1 - 341.41).abs() < 0.1, "{projected:?}");
    }

    #[test]
    fn table_local_point_to_world_plane_uses_reverse_projection_math() {
        let mut table_group = bitmap_group(1, "table", 137, 2, 365, 470);
        table_group.entries.push(EntryData {
            entry_type: FieldType::ShortValue,
            field_size: 4,
            payload: EntryPayload::RawBytes([200_i16, 0_i16].into_iter().flat_map(i16::to_le_bytes).collect()),
        });
        table_group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 12,
            payload: EntryPayload::RawBytes(
                [700.0_f32, 183.0, 238.0]
                    .into_iter()
                    .flat_map(f32::to_le_bytes)
                    .collect(),
            ),
        });

        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![
                raw_float_group(
                    0,
                    "camera_info",
                    &[
                        1.0, 0.0, 0.0, 0.0,
                        0.0, -0.913545, 0.406737, 3.791398,
                        0.0, -0.406737, -0.913545, 24.675402,
                        -400.000702, 19.501307, 4303.969727,
                    ],
                ),
                table_group,
            ],
        };

        let point = dat
            .table_local_point_to_world_plane(0, 182.50, 341.41, 0.0)
            .expect("reverse projection should succeed");
        assert!((point.0 - -7.020939).abs() < 0.1, "{point:?}");
        assert!((point.1 - 10.084854).abs() < 0.1, "{point:?}");
    }

    #[test]
    fn visual_collision_edges_decode_polygon_loops() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group_with_float_attribute(
                "wall",
                &[603.0, 4.0, 0.0, 0.0, 10.0, 0.0, 10.0, 10.0],
            )],
        };

        let edges = dat
            .visual_collision_edges(0, 0, 603, 0.0)
            .expect("polygon wall should decode");
        assert_eq!(edges.len(), 3);
        assert!(edges.iter().all(|edge| matches!(edge, VisualCollisionEdge::Line { .. })));
    }

    #[test]
    fn visual_collision_metadata_keeps_default_collision_group() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group_with_float_attribute("wall", &[600.0, 2.0, 10.0, 20.0, 30.0, 40.0])],
        };

        let metadata = dat
            .visual_collision_metadata(0, 0)
            .unwrap_or(VisualCollisionMetadata::default());
        assert_eq!(metadata.collision_group, 1);
    }

    #[test]
    fn visual_primary_points_decode_visual_line_payload() {
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group_with_float_attribute("oneway", &[600.0, 2.0, 30.0, 40.0, 10.0, 20.0])],
        };

        let points = dat
            .visual_primary_points(0, 0)
            .expect("visual points should decode");
        assert_eq!(points, vec![(30.0, 40.0), (10.0, 20.0)]);
    }

    #[test]
    fn visual_circle_attribute_306_decodes_circle_payload() {
        let mut group = group_with_float_attribute("kickout", &[600.0, 1.0, 30.0, 40.0, 5.0]);
        group.entries.push(EntryData {
            entry_type: FieldType::FloatArray,
            field_size: 8,
            payload: EntryPayload::RawBytes([306.0_f32, 2.0].into_iter().flat_map(f32::to_le_bytes).collect()),
        });
        let dat = DatFile {
            app_name: "test".to_string(),
            description: String::new(),
            groups: vec![group],
        };

        let circle = dat
            .visual_circle_attribute_306(0, 0)
            .expect("circle payload should decode");
        assert_eq!(
            circle,
            VisualCollisionEdge::Circle {
                center: (30.0, 40.0),
                radius: 10.0
            }
        );
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
