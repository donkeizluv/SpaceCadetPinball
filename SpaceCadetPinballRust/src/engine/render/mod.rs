pub mod scene;
pub mod sprite;
pub mod texture_cache;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

use crate::assets::DatFile;
use crate::engine::geom::RectI;
use crate::gameplay::{
    BitmapVisualState, LightVisualState, NumberWidgetVisualState, SequenceVisualState, TableVisual,
    TableVisualState, TextBoxVisualState,
};

use self::scene::RenderScene;
use self::sprite::SpriteRecord;
use self::texture_cache::TextureCache;

const MESSAGE_FONT_GROUP_NAME: &str = "pbmsg_ft";

#[derive(Debug, Clone, Copy)]
struct TextLayoutLine {
    start: usize,
    end: usize,
    width: i32,
}

pub struct RenderState {
    texture_cache: TextureCache,
    debug_state: RenderDebugState,
}

#[derive(Debug, Clone, Default)]
struct RenderDebugState {
    entries: Vec<String>,
}

impl RenderState {
    pub fn new() -> Self {
        Self {
            texture_cache: TextureCache::new(),
            debug_state: RenderDebugState::default(),
        }
    }

    pub fn begin_debug_frame(&mut self) {
        self.debug_state.entries.clear();
    }

    pub fn debug_summary(&self) -> Option<String> {
        if self.debug_state.entries.is_empty() {
            None
        } else {
            Some(self.debug_state.entries.join(" | "))
        }
    }

    pub fn draw_dat_file(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: &DatFile,
        resolution: usize,
        asset_revision: u64,
    ) -> Result<usize, String> {
        let scene = RenderScene::from_dat_file(dat_file, resolution, asset_revision);
        self.texture_cache.prune(scene.asset_revision);

        if let Some(sprite) = scene.sprites.first() {
            let group_name = dat_file
                .group_label(sprite.key.group_index)
                .unwrap_or("<unnamed>");
            self.record_debug_entry(format!("scene:{group_name}#{}", sprite.key.group_index));
        }

        let texture_creator = canvas.texture_creator();
        for sprite in &scene.sprites {
            self.texture_cache.draw_sprite(
                canvas,
                &texture_creator,
                dat_file,
                sprite,
                scene.asset_revision,
            )?;
        }

        Ok(scene.sprites.len())
    }

    pub fn draw_table_visuals(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: Option<&DatFile>,
        resolution: usize,
        asset_revision: u64,
        visuals: TableVisualState,
    ) -> Result<(), String> {
        self.draw_visual_list(
            canvas,
            dat_file,
            resolution,
            asset_revision,
            &visuals.visuals,
        )?;

        self.record_debug_entry(format!(
            "hud:score={} balls={} player={}",
            visuals.hud.score_value, visuals.hud.ball_count, visuals.hud.player_number
        ));

        Ok(())
    }

    fn draw_visual_list(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: Option<&DatFile>,
        resolution: usize,
        asset_revision: u64,
        visuals: &[TableVisual],
    ) -> Result<(), String> {
        for visual in visuals {
            self.draw_visual(canvas, dat_file, resolution, asset_revision, visual.clone())?;
        }

        Ok(())
    }

    fn draw_visual(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: Option<&DatFile>,
        resolution: usize,
        asset_revision: u64,
        visual: TableVisual,
    ) -> Result<(), String> {
        match visual {
            TableVisual::Bitmap(sprite_visual) => {
                self.draw_bitmap_visual(canvas, dat_file, resolution, asset_revision, sprite_visual)
            }
            TableVisual::Light(light) => {
                let Some(dat_file) = dat_file else {
                    return Ok(());
                };
                self.draw_light_visual(canvas, dat_file, resolution, asset_revision, light)
            }
            TableVisual::NumberWidget(widget) => {
                let Some(dat_file) = dat_file else {
                    return Ok(());
                };
                self.draw_number_widget(canvas, dat_file, resolution, asset_revision, widget)
            }
            TableVisual::Sequence(selection) => {
                let Some(dat_file) = dat_file else {
                    return Ok(());
                };
                self.draw_named_sequence_sprite(
                    canvas,
                    dat_file,
                    resolution,
                    asset_revision,
                    selection,
                )
            }
            TableVisual::TextBox(text_box) => {
                let Some(dat_file) = dat_file else {
                    return Ok(());
                };
                self.draw_text_box_visual(canvas, dat_file, resolution, asset_revision, text_box)
            }
        }
    }

    fn draw_bitmap_visual(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: Option<&DatFile>,
        resolution: usize,
        asset_revision: u64,
        sprite_visual: BitmapVisualState,
    ) -> Result<(), String> {
        if let Some(dat_file) = dat_file
            && let Some(bitmap_frame) =
                dat_file.named_bitmap_frame(sprite_visual.group_name, resolution)
        {
            let mut dest = sprite_visual.dest;
            if (sprite_visual.use_native_position || sprite_visual.use_native_size)
                && let Some(bitmap) =
                    dat_file.get_bitmap(bitmap_frame.group_index, bitmap_frame.bitmap_resolution)
            {
                if sprite_visual.use_native_position {
                    dest.x = i32::from(bitmap.x_position);
                    dest.y = i32::from(bitmap.y_position);
                }
                if sprite_visual.use_native_size {
                    dest.width = bitmap.width as u32;
                    dest.height = bitmap.height as u32;
                }
            }
            let sprite = SpriteRecord::at_dest(
                bitmap_frame.group_index,
                bitmap_frame.bitmap_resolution,
                dest,
                sprite_visual.depth_hint,
            );
            let texture_creator = canvas.texture_creator();
            self.texture_cache.draw_sprite(
                canvas,
                &texture_creator,
                dat_file,
                &sprite,
                asset_revision,
            )?;
            self.record_debug_entry(format!(
                "bitmap:{}#{}",
                sprite_visual.group_name, bitmap_frame.group_index
            ));
            return Ok(());
        }

        self.record_debug_entry(format!("bitmap:{}:overlay", sprite_visual.group_name));
        self.draw_bitmap_overlay(canvas, sprite_visual)
    }

    fn draw_text_box_visual(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: &DatFile,
        resolution: usize,
        asset_revision: u64,
        text_box: TextBoxVisualState,
    ) -> Result<(), String> {
        let Some(layout) = dat_file.text_box_layout(text_box.group_name) else {
            self.record_debug_entry(format!("{}:layout-missing", text_box.group_name));
            return Ok(());
        };

        let bounds = RectI::new(layout.x, layout.y, layout.width, layout.height);
        let previous_clip = canvas.clip_rect();
        canvas.set_clip_rect(Some(bounds.to_sdl_rect()));

        let used_bitmap_font = self.draw_bitmap_text_box(
            canvas,
            dat_file,
            resolution,
            asset_revision,
            bounds,
            &text_box.text,
        )?;
        if !used_bitmap_font {
            self.draw_pixel_text_box(canvas, bounds, &text_box.text)?;
        }

        canvas.set_clip_rect(previous_clip);
        self.record_debug_entry(format!("textbox:{}", text_box.group_name));
        Ok(())
    }

    fn draw_light_visual(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: &DatFile,
        resolution: usize,
        asset_revision: u64,
        light: LightVisualState,
    ) -> Result<(), String> {
        self.draw_named_sequence_sprite(
            canvas,
            dat_file,
            resolution,
            asset_revision,
            SequenceVisualState {
                group_name: light.group_name,
                frame_fraction: light.frame_fraction,
            },
        )
    }

    fn draw_number_widget(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: &DatFile,
        resolution: usize,
        asset_revision: u64,
        widget: NumberWidgetVisualState,
    ) -> Result<(), String> {
        let Some(layout) = dat_file.hud_widget_layout(widget.widget_group_name) else {
            self.record_debug_entry(format!("{}:layout-missing", widget.widget_group_name));
            return Ok(());
        };
        let Some(digit_groups) = dat_file.number_widget_digit_groups(
            widget.font_group_name,
            resolution,
            widget.value,
            layout.digits,
        ) else {
            self.record_debug_entry(format!("{}:missing", widget.font_group_name));
            return Ok(());
        };

        let digit_count = digit_groups.len().max(1);
        let bounds = RectI::new(layout.x, layout.y, layout.width, layout.height);
        let cell_width = (bounds.width / layout.digits.max(1) as u32).max(1);
        let start_x = bounds.right() - (digit_count as i32 * cell_width as i32);
        let texture_creator = canvas.texture_creator();

        for (index, group_index) in digit_groups.into_iter().enumerate() {
            let Some(group) = dat_file.group(group_index) else {
                continue;
            };
            let Some(bitmap) = group.get_bitmap(resolution) else {
                continue;
            };

            let dest = RectI::new(
                start_x + index as i32 * cell_width as i32,
                bounds.y,
                cell_width,
                bounds.height,
            );
            let sprite = SpriteRecord::at_dest(group_index, bitmap.resolution, dest, dest.y);
            self.texture_cache.draw_sprite(
                canvas,
                &texture_creator,
                dat_file,
                &sprite,
                asset_revision,
            )?;
        }

        Ok(())
    }

    fn draw_pixel_text_box(
        &mut self,
        canvas: &mut Canvas<Window>,
        bounds: RectI,
        text: &str,
    ) -> Result<(), String> {
        const GLYPH_HEIGHT: i32 = 7;
        const GLYPH_ADVANCE: i32 = 6;
        const LINE_HEIGHT: i32 = 8;

        let max_columns = (bounds.width as i32 / GLYPH_ADVANCE).max(1) as usize;
        let max_lines = (bounds.height as i32 / LINE_HEIGHT).max(1) as usize;
        let lines = wrap_text_lines(text, max_columns, max_lines);
        if lines.is_empty() {
            return Ok(());
        }

        let text_height = (lines.len() as i32 * LINE_HEIGHT).max(GLYPH_HEIGHT);
        let mut cursor_y = bounds.y + ((bounds.height as i32 - text_height).max(0) / 2);

        for line in &lines {
            let line_width = (line.chars().count() as i32 * GLYPH_ADVANCE)
                .saturating_sub(1)
                .max(0);
            let mut cursor_x = bounds.x + ((bounds.width as i32 - line_width).max(0) / 2);

            for character in line.chars() {
                draw_pixel_glyph(
                    canvas,
                    cursor_x,
                    cursor_y,
                    character.to_ascii_uppercase(),
                    Color::RGB(255, 210, 120),
                )?;
                cursor_x += GLYPH_ADVANCE;
            }

            cursor_y += LINE_HEIGHT;
        }

        Ok(())
    }

    fn draw_bitmap_text_box(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: &DatFile,
        resolution: usize,
        asset_revision: u64,
        bounds: RectI,
        text: &str,
    ) -> Result<bool, String> {
        let Some(font) = dat_file.message_font(MESSAGE_FONT_GROUP_NAME, resolution) else {
            return Ok(false);
        };

        let lines =
            layout_message_font_lines(text, &font, bounds.width as i32, bounds.height as i32);
        if lines.is_empty() {
            return Ok(true);
        }

        let text_height = lines.len() as i32 * font.line_height;
        let mut offset_y = bounds.y + ((bounds.height as i32 - text_height).max(0) / 2);
        let texture_creator = canvas.texture_creator();
        let bytes = text.as_bytes();

        for line in &lines {
            let mut offset_x = bounds.x + ((bounds.width as i32 - line.width).max(0) / 2);

            for &byte in &bytes[line.start..line.end] {
                let masked = byte & 0x7F;
                let Some(glyph) = font.glyph(masked) else {
                    continue;
                };

                let dest = RectI::new(offset_x, offset_y, glyph.width, glyph.height);
                let sprite = SpriteRecord::at_dest(
                    glyph.group_index,
                    glyph.bitmap_resolution,
                    dest,
                    offset_y,
                );
                self.texture_cache.draw_sprite(
                    canvas,
                    &texture_creator,
                    dat_file,
                    &sprite,
                    asset_revision,
                )?;
                offset_x += glyph.width as i32 + font.gap_width;
            }

            offset_y += font.line_height;
        }

        Ok(true)
    }

    fn draw_named_sequence_sprite(
        &mut self,
        canvas: &mut Canvas<Window>,
        dat_file: &DatFile,
        resolution: usize,
        asset_revision: u64,
        selection: SequenceVisualState,
    ) -> Result<(), String> {
        let Some(frame) =
            dat_file.sequence_frame(selection.group_name, resolution, selection.frame_fraction)
        else {
            self.record_debug_entry(format!("{}:missing", selection.group_name));
            return Ok(());
        };
        let group_index = frame.group_index;
        let Some(group) = dat_file.group(group_index) else {
            return Ok(());
        };
        let Some(bitmap) = group.get_bitmap(resolution) else {
            return Ok(());
        };

        let (table_origin_x, table_origin_y) =
            dat_file.table_bitmap_origin(resolution).unwrap_or((0, 0));
        let dest = RectI::new(
            i32::from(bitmap.x_position) - table_origin_x,
            i32::from(bitmap.y_position) - table_origin_y,
            bitmap.width as u32,
            bitmap.height as u32,
        );

        let sprite = SpriteRecord::at_dest(group_index, bitmap.resolution, dest, dest.y);
        let texture_creator = canvas.texture_creator();
        self.texture_cache.draw_sprite(
            canvas,
            &texture_creator,
            dat_file,
            &sprite,
            asset_revision,
        )?;

        self.record_debug_entry(format!(
            "{}:{}/{}#{}",
            selection.group_name,
            frame.frame_index + 1,
            frame.frame_count,
            group_index
        ));

        Ok(())
    }

    fn record_debug_entry(&mut self, entry: String) {
        self.debug_state.entries.push(entry);
    }

    fn draw_bitmap_overlay(
        &mut self,
        canvas: &mut Canvas<Window>,
        sprite_visual: BitmapVisualState,
    ) -> Result<(), String> {
        let bounds = sprite_visual.dest;
        let shade = sprite_visual.fallback_shade;
        canvas.set_draw_color(Color::RGB(shade, shade, shade));
        canvas
            .fill_rect(Rect::new(bounds.x, bounds.y, bounds.width, bounds.height))
            .map_err(|error| error.to_string())
    }

    pub fn draw_status_overlay(
        &mut self,
        canvas: &mut Canvas<Window>,
        color: Color,
    ) -> Result<(), String> {
        canvas.set_draw_color(color);
        canvas
            .fill_rect(Rect::new(16, 16, 96, 96))
            .map_err(|error| error.to_string())
    }
}

fn wrap_text_lines(text: &str, max_columns: usize, max_lines: usize) -> Vec<String> {
    let mut lines = Vec::new();

    for paragraph in text.split('\n') {
        let mut current = String::new();
        for word in paragraph.split_whitespace() {
            let candidate_len = if current.is_empty() {
                word.len()
            } else {
                current.len() + 1 + word.len()
            };

            if candidate_len > max_columns && !current.is_empty() {
                lines.push(current);
                if lines.len() >= max_lines {
                    return lines;
                }
                current = word.to_string();
            } else {
                if !current.is_empty() {
                    current.push(' ');
                }
                current.push_str(word);
            }
        }

        if !current.is_empty() {
            lines.push(current);
            if lines.len() >= max_lines {
                return lines;
            }
        }
    }

    lines
}

fn layout_message_font_lines(
    text: &str,
    font: &crate::assets::MessageFont,
    max_width: i32,
    max_height: i32,
) -> Vec<TextLayoutLine> {
    let bytes = text.as_bytes();
    let line_height = font.line_height.max(1);
    let max_lines = (max_height.max(line_height) / line_height).max(1) as usize;
    let mut lines = Vec::new();
    let mut start = 0usize;

    while start < bytes.len() && lines.len() < max_lines {
        let Some((line, next_start)) = layout_message_font_line(bytes, start, font, max_width)
        else {
            break;
        };
        if line.start == line.end {
            break;
        }
        lines.push(line);
        start = next_start;
    }

    lines
}

fn layout_message_font_line(
    bytes: &[u8],
    start: usize,
    font: &crate::assets::MessageFont,
    max_width: i32,
) -> Option<(TextLayoutLine, usize)> {
    let mut line_width = 0i32;
    let mut word_width = 0i32;
    let mut word_boundary = None;
    let mut text_end = start;

    while text_end < bytes.len() {
        let masked = bytes[text_end] & 0x7F;
        if masked == 0 || masked == b'\n' {
            break;
        }

        let Some(glyph) = font.glyph(masked) else {
            text_end += 1;
            continue;
        };

        let width = line_width + glyph.width as i32 + font.gap_width;
        if width > max_width {
            if let Some(boundary) = word_boundary {
                text_end = boundary;
                line_width = word_width;
            }
            break;
        }

        if masked == b' ' {
            word_boundary = Some(text_end);
            word_width = width;
        }
        line_width = width;
        text_end += 1;
    }

    let draw_end = text_end;
    while text_end < bytes.len() && (bytes[text_end] & 0x7F) == b' ' {
        text_end += 1;
    }
    if text_end < bytes.len() && (bytes[text_end] & 0x7F) == b'\n' {
        text_end += 1;
    }

    Some((
        TextLayoutLine {
            start,
            end: draw_end,
            width: line_width.max(0),
        },
        text_end,
    ))
}

fn draw_pixel_glyph(
    canvas: &mut Canvas<Window>,
    x: i32,
    y: i32,
    character: char,
    color: Color,
) -> Result<(), String> {
    let glyph = glyph_rows(character);
    canvas.set_draw_color(color);

    for (row_index, row_bits) in glyph.iter().enumerate() {
        for column in 0..5 {
            if (row_bits >> (4 - column)) & 1 == 1 {
                canvas.fill_rect(Rect::new(x + column, y + row_index as i32, 1, 1))?;
            }
        }
    }

    Ok(())
}

fn glyph_rows(character: char) -> [u8; 7] {
    match character {
        'A' => [0x0E, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'B' => [0x1E, 0x11, 0x11, 0x1E, 0x11, 0x11, 0x1E],
        'C' => [0x0E, 0x11, 0x10, 0x10, 0x10, 0x11, 0x0E],
        'D' => [0x1E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x1E],
        'E' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x1F],
        'F' => [0x1F, 0x10, 0x10, 0x1E, 0x10, 0x10, 0x10],
        'G' => [0x0E, 0x11, 0x10, 0x17, 0x11, 0x11, 0x0E],
        'H' => [0x11, 0x11, 0x11, 0x1F, 0x11, 0x11, 0x11],
        'I' => [0x0E, 0x04, 0x04, 0x04, 0x04, 0x04, 0x0E],
        'J' => [0x01, 0x01, 0x01, 0x01, 0x11, 0x11, 0x0E],
        'K' => [0x11, 0x12, 0x14, 0x18, 0x14, 0x12, 0x11],
        'L' => [0x10, 0x10, 0x10, 0x10, 0x10, 0x10, 0x1F],
        'M' => [0x11, 0x1B, 0x15, 0x15, 0x11, 0x11, 0x11],
        'N' => [0x11, 0x11, 0x19, 0x15, 0x13, 0x11, 0x11],
        'O' => [0x0E, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'P' => [0x1E, 0x11, 0x11, 0x1E, 0x10, 0x10, 0x10],
        'Q' => [0x0E, 0x11, 0x11, 0x11, 0x15, 0x12, 0x0D],
        'R' => [0x1E, 0x11, 0x11, 0x1E, 0x14, 0x12, 0x11],
        'S' => [0x0F, 0x10, 0x10, 0x0E, 0x01, 0x01, 0x1E],
        'T' => [0x1F, 0x04, 0x04, 0x04, 0x04, 0x04, 0x04],
        'U' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x0E],
        'V' => [0x11, 0x11, 0x11, 0x11, 0x11, 0x0A, 0x04],
        'W' => [0x11, 0x11, 0x11, 0x15, 0x15, 0x15, 0x0A],
        'X' => [0x11, 0x11, 0x0A, 0x04, 0x0A, 0x11, 0x11],
        'Y' => [0x11, 0x11, 0x0A, 0x04, 0x04, 0x04, 0x04],
        'Z' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x10, 0x1F],
        '0' => [0x0E, 0x11, 0x13, 0x15, 0x19, 0x11, 0x0E],
        '1' => [0x04, 0x0C, 0x04, 0x04, 0x04, 0x04, 0x0E],
        '2' => [0x0E, 0x11, 0x01, 0x02, 0x04, 0x08, 0x1F],
        '3' => [0x1F, 0x02, 0x04, 0x02, 0x01, 0x11, 0x0E],
        '4' => [0x02, 0x06, 0x0A, 0x12, 0x1F, 0x02, 0x02],
        '5' => [0x1F, 0x10, 0x1E, 0x01, 0x01, 0x11, 0x0E],
        '6' => [0x06, 0x08, 0x10, 0x1E, 0x11, 0x11, 0x0E],
        '7' => [0x1F, 0x01, 0x02, 0x04, 0x08, 0x08, 0x08],
        '8' => [0x0E, 0x11, 0x11, 0x0E, 0x11, 0x11, 0x0E],
        '9' => [0x0E, 0x11, 0x11, 0x0F, 0x01, 0x02, 0x0C],
        '%' => [0x19, 0x19, 0x02, 0x04, 0x08, 0x13, 0x13],
        ':' => [0x00, 0x04, 0x04, 0x00, 0x04, 0x04, 0x00],
        '.' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x0C],
        '!' => [0x04, 0x04, 0x04, 0x04, 0x04, 0x00, 0x04],
        '-' => [0x00, 0x00, 0x00, 0x1F, 0x00, 0x00, 0x00],
        '/' => [0x01, 0x02, 0x02, 0x04, 0x08, 0x08, 0x10],
        ' ' => [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00],
        _ => [0x1F, 0x01, 0x02, 0x04, 0x04, 0x00, 0x04],
    }
}
