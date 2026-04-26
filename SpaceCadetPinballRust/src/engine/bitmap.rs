const RESOLUTION_TABLE_WIDTHS: [usize; 3] = [600, 752, 960];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapType {
    RawBitmap,
    DibBitmap,
    Spliced,
}

#[derive(Debug, Clone)]
pub struct Bitmap8 {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub indexed_stride: usize,
    pub x_position: i16,
    pub y_position: i16,
    pub resolution: usize,
    pub bitmap_type: BitmapType,
    pub indexed_pixels: Vec<u8>,
}

impl Bitmap8 {
    pub fn average_color_index(&self) -> u8 {
        let mut total = 0u64;
        let mut count = 0u64;

        for row in 0..self.height {
            let start = row * self.indexed_stride;
            let end = start + self.width;
            for sample in &self.indexed_pixels[start..end] {
                total += u64::from(*sample);
                count += 1;
            }
        }

        if count == 0 { 0 } else { (total / count) as u8 }
    }
}

#[derive(Debug, Clone)]
pub struct ZMap {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub resolution: usize,
    pub samples: Vec<u16>,
}

impl ZMap {
    pub fn average_sample(&self) -> u16 {
        if self.samples.is_empty() {
            return 0;
        }

        let total = self
            .samples
            .iter()
            .fold(0u64, |sum, sample| sum + u64::from(*sample));
        (total / self.samples.len() as u64) as u16
    }
}

pub fn resolution_table_width(resolution: usize) -> Option<usize> {
    RESOLUTION_TABLE_WIDTHS.get(resolution).copied()
}
