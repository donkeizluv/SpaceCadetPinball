use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;

const DAT_SIGNATURE: &[u8; 20] = b"PARTOUT(4.0)RESOURCE";
const FIELD_SIZES: [i32; 14] = [2, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0];
const RESOLUTION_TABLE_WIDTHS: [usize; 3] = [600, 752, 960];

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[repr(u8)]
pub enum FieldType {
    ShortValue = 0,
    Bitmap8bit = 1,
    Unknown2 = 2,
    GroupName = 3,
    Unknown4 = 4,
    Palette = 5,
    Unknown6 = 6,
    Unknown7 = 7,
    Unknown8 = 8,
    String = 9,
    ShortArray = 10,
    FloatArray = 11,
    Bitmap16bit = 12,
}

impl FieldType {
    fn from_raw(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::ShortValue),
            1 => Some(Self::Bitmap8bit),
            2 => Some(Self::Unknown2),
            3 => Some(Self::GroupName),
            4 => Some(Self::Unknown4),
            5 => Some(Self::Palette),
            6 => Some(Self::Unknown6),
            7 => Some(Self::Unknown7),
            8 => Some(Self::Unknown8),
            9 => Some(Self::String),
            10 => Some(Self::ShortArray),
            11 => Some(Self::FloatArray),
            12 => Some(Self::Bitmap16bit),
            _ => None,
        }
    }

    fn as_index(self) -> usize {
        self as usize
    }
}

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

#[derive(Debug, Clone)]
pub struct ZMap {
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub resolution: usize,
    pub samples: Vec<u16>,
}

#[derive(Debug, Clone)]
pub enum EntryPayload {
    RawBytes(Vec<u8>),
    Text(String),
    Bitmap8(Bitmap8),
    Bitmap16(ZMap),
    Bitmap8Ref(usize),
    Bitmap16Ref(usize),
}

#[derive(Debug, Clone)]
pub struct EntryData {
    pub entry_type: FieldType,
    pub field_size: i32,
    pub payload: EntryPayload,
}

#[derive(Debug, Clone)]
pub struct GroupData {
    pub group_id: usize,
    pub group_name: Option<String>,
    pub entries: Vec<EntryData>,
    bitmaps: [Option<Bitmap8>; 3],
    zmaps: [Option<ZMap>; 3],
    needs_sort: bool,
}

impl GroupData {
    fn new(group_id: usize) -> Self {
        Self {
            group_id,
            group_name: None,
            entries: Vec::new(),
            bitmaps: [None, None, None],
            zmaps: [None, None, None],
            needs_sort: false,
        }
    }

    fn add_entry(&mut self, entry: EntryData) -> Result<(), String> {
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

    fn finalize_group(&mut self) {
        if self.needs_sort {
            self.needs_sort = false;
            self.entries.sort_by_key(|entry| entry.entry_type as u8);
            self.entries.shrink_to_fit();
        }
    }

    pub fn get_bitmap(&self, resolution: usize) -> Option<&Bitmap8> {
        self.bitmaps.get(resolution).and_then(Option::as_ref)
    }

    pub fn get_zmap(&self, resolution: usize) -> Option<&ZMap> {
        self.zmaps.get(resolution).and_then(Option::as_ref)
    }
}

#[derive(Debug, Clone)]
pub struct DatFile {
    pub app_name: String,
    pub description: String,
    pub groups: Vec<GroupData>,
}

impl DatFile {
    fn finalize(&mut self) {
        for group in &mut self.groups {
            group.finalize_group();
        }
    }

    pub fn record_labeled(&self, target_group_name: &str) -> Option<usize> {
        self.groups
            .iter()
            .rposition(|group| group.group_name.as_deref() == Some(target_group_name))
    }

    pub fn get_bitmap(&self, group_index: usize, resolution: usize) -> Option<&Bitmap8> {
        self.groups.get(group_index)?.get_bitmap(resolution)
    }

    pub fn get_zmap(&self, group_index: usize, resolution: usize) -> Option<&ZMap> {
        self.groups.get(group_index)?.get_zmap(resolution)
    }
}

#[derive(Debug)]
struct DatFileHeader {
    app_name: String,
    description: String,
    number_of_groups: u16,
    unknown_size: u16,
}

#[derive(Debug)]
struct Dat8BitBmpHeader {
    resolution: u8,
    width: i16,
    height: i16,
    x_position: i16,
    y_position: i16,
    size: i32,
    flags: u8,
}

impl Dat8BitBmpHeader {
    fn is_flag_set(&self, flag: u8) -> bool {
        (self.flags & flag) != 0
    }
}

#[derive(Debug)]
struct Dat16BitBmpHeader {
    width: i16,
    height: i16,
    stride: i16,
    _unknown0: i32,
    _unknown1_0: i16,
    _unknown1_1: i16,
}

pub fn load_records(path: &Path, full_tilt_mode: bool) -> Result<DatFile, String> {
    let mut file = File::open(path)
        .map_err(|error| format!("failed to open DAT file {}: {error}", path.display()))?;

    let header = read_dat_header(&mut file)?;
    if header.unknown_size > 0 {
        let mut unknown = vec![0u8; header.unknown_size as usize];
        file.read_exact(&mut unknown)
            .map_err(|error| format!("failed to read DAT unknown header bytes: {error}"))?;
    }

    let mut dat_file = DatFile {
        app_name: header.app_name,
        description: header.description,
        groups: Vec::with_capacity(header.number_of_groups as usize),
    };

    for group_index in 0..header.number_of_groups as usize {
        let entry_count = read_u8(&mut file)
            .map_err(|error| format!("failed to read group entry count: {error}"))?;
        let mut group = GroupData::new(group_index);

        for _ in 0..entry_count as usize {
            let raw_type = read_u8(&mut file).map_err(|error| {
                format!("failed to read entry type for group {group_index}: {error}")
            })?;
            let field_type = FieldType::from_raw(raw_type).ok_or_else(|| {
                format!("group {group_index}: unsupported field type id {raw_type}")
            })?;

            let mut field_size = FIELD_SIZES
                .get(field_type.as_index())
                .copied()
                .ok_or_else(|| format!("group {group_index}: invalid field type index"))?;
            if field_size < 0 {
                let variable_size = read_u32(&mut file)
                    .map_err(|error| format!("failed to read variable field size: {error}"))?;
                field_size = i32::try_from(variable_size)
                    .map_err(|_| "field size does not fit in i32".to_string())?;
            }

            if field_size < 0 {
                return Err(format!(
                    "group {group_index}: negative field size {field_size} for type {raw_type}"
                ));
            }

            let entry = match field_type {
                FieldType::Bitmap8bit => {
                    let bmp_header = read_bitmap8_header(&mut file)?;
                    let payload_size = field_size.checked_sub(14).ok_or_else(|| {
                        format!("group {group_index}: invalid bitmap8 field size")
                    })?;
                    let payload = read_exact_vec(&mut file, payload_size as usize)
                        .map_err(|error| format!("failed to read bitmap8 payload: {error}"))?;
                    let bitmap = parse_bitmap8(&bmp_header, payload)?;
                    EntryData {
                        entry_type: field_type,
                        field_size,
                        payload: EntryPayload::Bitmap8(bitmap),
                    }
                }
                FieldType::Bitmap16bit => {
                    let zmap_resolution = if full_tilt_mode {
                        field_size -= 1;
                        usize::from(read_u8(&mut file).map_err(|error| {
                            format!("failed to read Full Tilt zMap resolution byte: {error}")
                        })?)
                    } else {
                        0
                    };

                    let zmap_header = read_bitmap16_header(&mut file)?;
                    let payload_size = field_size.checked_sub(14).ok_or_else(|| {
                        format!("group {group_index}: invalid bitmap16 field size")
                    })?;
                    let payload = read_exact_vec(&mut file, payload_size as usize)
                        .map_err(|error| format!("failed to read bitmap16 payload: {error}"))?;
                    let zmap = parse_zmap(&zmap_header, zmap_resolution, payload);
                    EntryData {
                        entry_type: field_type,
                        field_size,
                        payload: EntryPayload::Bitmap16(zmap),
                    }
                }
                FieldType::GroupName | FieldType::String => {
                    let bytes = read_exact_vec(&mut file, field_size as usize)
                        .map_err(|error| format!("failed to read text field payload: {error}"))?;
                    let text = clean_c_string(&bytes);
                    EntryData {
                        entry_type: field_type,
                        field_size,
                        payload: EntryPayload::Text(text),
                    }
                }
                _ => {
                    let bytes = read_exact_vec(&mut file, field_size as usize)
                        .map_err(|error| format!("failed to read field payload: {error}"))?;
                    EntryData {
                        entry_type: field_type,
                        field_size,
                        payload: EntryPayload::RawBytes(bytes),
                    }
                }
            };

            group.add_entry(entry)?;
        }

        dat_file.groups.push(group);
    }

    dat_file.finalize();
    Ok(dat_file)
}

fn parse_bitmap8(header: &Dat8BitBmpHeader, payload: Vec<u8>) -> Result<Bitmap8, String> {
    let width = i32::from(header.width);
    let height = i32::from(header.height);
    if width < 0 || height < 0 {
        return Err("bitmap8 has negative dimensions".to_string());
    }

    let width = width as usize;
    let height = height as usize;
    let mut indexed_stride = width;

    const RAW_BMP_UNALIGNED: u8 = 1 << 0;
    const DIB_BITMAP: u8 = 1 << 1;
    const SPLICED: u8 = 1 << 2;

    let bitmap_type = if header.is_flag_set(SPLICED) {
        BitmapType::Spliced
    } else if header.is_flag_set(DIB_BITMAP) {
        BitmapType::DibBitmap
    } else {
        BitmapType::RawBitmap
    };

    let expected_size = if bitmap_type == BitmapType::Spliced {
        usize::try_from(header.size).map_err(|_| "bitmap8 size overflow".to_string())?
    } else {
        if width % 4 != 0 {
            if bitmap_type == BitmapType::RawBitmap && !header.is_flag_set(RAW_BMP_UNALIGNED) {
                return Err("raw bitmap missing alignment flag".to_string());
            }
            indexed_stride = width + (4 - (width % 4));
        }
        indexed_stride * height
    };

    if payload.len() != expected_size {
        return Err(format!(
            "bitmap8 payload size mismatch, expected {}, got {}",
            expected_size,
            payload.len()
        ));
    }

    let resolution = usize::from(header.resolution);
    if resolution > 2 {
        return Err(format!("bitmap8 resolution {} out of bounds", resolution));
    }

    Ok(Bitmap8 {
        width,
        height,
        stride: width,
        indexed_stride,
        x_position: header.x_position,
        y_position: header.y_position,
        resolution,
        bitmap_type,
        indexed_pixels: payload,
    })
}

fn parse_zmap(header: &Dat16BitBmpHeader, resolution: usize, payload: Vec<u8>) -> ZMap {
    let width = i32::from(header.width);
    let height = i32::from(header.height);
    let stride = i32::from(header.stride);

    let is_valid = width >= 0
        && height >= 0
        && stride >= 0
        && usize::try_from(stride)
            .ok()
            .unwrap_or(0)
            .saturating_mul(usize::try_from(height).ok().unwrap_or(0))
            .saturating_mul(2)
            == payload.len();

    if !is_valid {
        return ZMap {
            width: 0,
            height: 0,
            stride: 0,
            resolution,
            samples: Vec::new(),
        };
    }

    let width = width as usize;
    let height = height as usize;
    let stride = stride as usize;

    let mut samples = Vec::with_capacity(payload.len() / 2);
    for chunk in payload.chunks_exact(2) {
        samples.push(u16::from_le_bytes([chunk[0], chunk[1]]));
    }

    ZMap {
        width,
        height,
        stride,
        resolution,
        samples,
    }
}

fn split_spliced_bitmap(source: &Bitmap8) -> Result<(Bitmap8, ZMap), String> {
    if source.bitmap_type != BitmapType::Spliced {
        return Err("split_spliced_bitmap called for non-spliced bitmap".to_string());
    }

    let table_width = *RESOLUTION_TABLE_WIDTHS
        .get(source.resolution)
        .ok_or_else(|| {
            format!(
                "unsupported spliced bitmap resolution {}",
                source.resolution
            )
        })?;

    let mut bitmap = Bitmap8 {
        width: source.width,
        height: source.height,
        stride: source.width,
        indexed_stride: source.width,
        x_position: source.x_position,
        y_position: source.y_position,
        resolution: source.resolution,
        bitmap_type: BitmapType::DibBitmap,
        indexed_pixels: vec![0xFF; source.width.saturating_mul(source.height)],
    };

    let mut zmap = ZMap {
        width: source.width,
        height: source.height,
        stride: source.width,
        resolution: source.resolution,
        samples: vec![0xFFFF; source.width.saturating_mul(source.height)],
    };

    let bytes = &source.indexed_pixels;
    let mut offset = 0usize;
    let mut dst_index = 0usize;

    loop {
        let stride = read_i16_from_slice(bytes, &mut offset)?;
        if stride < 0 {
            break;
        }

        let mut adjusted_stride = i32::from(stride);
        if adjusted_stride > bitmap.width as i32 {
            adjusted_stride += bitmap.width as i32 - table_width as i32;
            if adjusted_stride < 0 {
                return Err("spliced bitmap produced negative stride".to_string());
            }
        }

        dst_index = dst_index.saturating_add(adjusted_stride as usize);

        let count = read_u16_from_slice(bytes, &mut offset)?;
        for _ in 0..usize::from(count) {
            let depth = read_u16_from_slice(bytes, &mut offset)?;
            let color = *bytes
                .get(offset)
                .ok_or_else(|| "unexpected end of spliced bitmap data".to_string())?;
            offset += 1;

            if dst_index >= bitmap.indexed_pixels.len() || dst_index >= zmap.samples.len() {
                return Err("spliced bitmap decoded out of bounds".to_string());
            }

            bitmap.indexed_pixels[dst_index] = color;
            zmap.samples[dst_index] = depth;
            dst_index += 1;
        }
    }

    Ok((bitmap, zmap))
}

fn flip_zmap_horizontally(zmap: &mut ZMap) {
    if zmap.width == 0 || zmap.height == 0 || zmap.stride == 0 {
        return;
    }

    let mut top_row = 0usize;
    let mut bottom_row = zmap.height - 1;

    while bottom_row >= zmap.height / 2 {
        let top_start = top_row * zmap.stride;
        let bottom_start = bottom_row * zmap.stride;

        for x in 0..zmap.width {
            zmap.samples.swap(top_start + x, bottom_start + x);
        }

        top_row += 1;
        if bottom_row == 0 {
            break;
        }
        bottom_row -= 1;
    }
}

fn clean_fixed_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|&byte| byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

fn clean_c_string(bytes: &[u8]) -> String {
    clean_fixed_string(bytes)
}

fn read_dat_header<R: Read>(reader: &mut R) -> Result<DatFileHeader, String> {
    let mut signature = [0u8; 21];
    reader
        .read_exact(&mut signature)
        .map_err(|error| format!("failed to read DAT signature: {error}"))?;

    // TEST: no need to check
    // if &signature[..DAT_SIGNATURE.len()] != DAT_SIGNATURE {
    //     return Err("invalid DAT signature".to_string());
    // }

    let mut app_name = [0u8; 50];
    let mut description = [0u8; 100];
    reader
        .read_exact(&mut app_name)
        .map_err(|error| format!("failed to read DAT app name: {error}"))?;
    reader
        .read_exact(&mut description)
        .map_err(|error| format!("failed to read DAT description: {error}"))?;

    let _file_size =
        read_i32(reader).map_err(|error| format!("failed to read file_size: {error}"))?;
    let number_of_groups =
        read_u16(reader).map_err(|error| format!("failed to read number_of_groups: {error}"))?;
    let _size_of_body =
        read_i32(reader).map_err(|error| format!("failed to read body_size: {error}"))?;
    let unknown_size =
        read_u16(reader).map_err(|error| format!("failed to read unknown_size: {error}"))?;

    Ok(DatFileHeader {
        app_name: clean_fixed_string(&app_name),
        description: clean_fixed_string(&description),
        number_of_groups,
        unknown_size,
    })
}

fn read_bitmap8_header<R: Read>(reader: &mut R) -> Result<Dat8BitBmpHeader, String> {
    Ok(Dat8BitBmpHeader {
        resolution: read_u8(reader).map_err(|error| format!("bitmap8 resolution: {error}"))?,
        width: read_i16(reader).map_err(|error| format!("bitmap8 width: {error}"))?,
        height: read_i16(reader).map_err(|error| format!("bitmap8 height: {error}"))?,
        x_position: read_i16(reader).map_err(|error| format!("bitmap8 x_position: {error}"))?,
        y_position: read_i16(reader).map_err(|error| format!("bitmap8 y_position: {error}"))?,
        size: read_i32(reader).map_err(|error| format!("bitmap8 size: {error}"))?,
        flags: read_u8(reader).map_err(|error| format!("bitmap8 flags: {error}"))?,
    })
}

fn read_bitmap16_header<R: Read>(reader: &mut R) -> Result<Dat16BitBmpHeader, String> {
    Ok(Dat16BitBmpHeader {
        width: read_i16(reader).map_err(|error| format!("bitmap16 width: {error}"))?,
        height: read_i16(reader).map_err(|error| format!("bitmap16 height: {error}"))?,
        stride: read_i16(reader).map_err(|error| format!("bitmap16 stride: {error}"))?,
        _unknown0: read_i32(reader).map_err(|error| format!("bitmap16 unknown0: {error}"))?,
        _unknown1_0: read_i16(reader).map_err(|error| format!("bitmap16 unknown1_0: {error}"))?,
        _unknown1_1: read_i16(reader).map_err(|error| format!("bitmap16 unknown1_1: {error}"))?,
    })
}

fn read_u8<R: Read>(reader: &mut R) -> std::io::Result<u8> {
    let mut bytes = [0u8; 1];
    reader.read_exact(&mut bytes)?;
    Ok(bytes[0])
}

fn read_u16<R: Read>(reader: &mut R) -> std::io::Result<u16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(u16::from_le_bytes(bytes))
}

fn read_i16<R: Read>(reader: &mut R) -> std::io::Result<i16> {
    let mut bytes = [0u8; 2];
    reader.read_exact(&mut bytes)?;
    Ok(i16::from_le_bytes(bytes))
}

fn read_u32<R: Read>(reader: &mut R) -> std::io::Result<u32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}

fn read_i32<R: Read>(reader: &mut R) -> std::io::Result<i32> {
    let mut bytes = [0u8; 4];
    reader.read_exact(&mut bytes)?;
    Ok(i32::from_le_bytes(bytes))
}

fn read_exact_vec<R: Read + Seek>(reader: &mut R, size: usize) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; size];
    reader.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn read_u16_from_slice(bytes: &[u8], offset: &mut usize) -> Result<u16, String> {
    if bytes.len().saturating_sub(*offset) < 2 {
        return Err("unexpected end of spliced bitmap data".to_string());
    }
    let value = u16::from_le_bytes([bytes[*offset], bytes[*offset + 1]]);
    *offset += 2;
    Ok(value)
}

fn read_i16_from_slice(bytes: &[u8], offset: &mut usize) -> Result<i16, String> {
    if bytes.len().saturating_sub(*offset) < 2 {
        return Err("unexpected end of spliced bitmap data".to_string());
    }
    let value = i16::from_le_bytes([bytes[*offset], bytes[*offset + 1]]);
    *offset += 2;
    Ok(value)
}
