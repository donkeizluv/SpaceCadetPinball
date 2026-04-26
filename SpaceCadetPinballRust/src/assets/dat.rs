use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::engine::bitmap::{Bitmap8, BitmapType, ZMap};

const DAT_SIGNATURE: &[u8; 20] = b"PARTOUT(4.0)RESOURCE";
const FIELD_SIZES: [i32; 14] = [2, -1, 2, -1, -1, -1, -1, -1, -1, -1, -1, -1, -1, 0];
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
        match value & 0x0F {
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
    pub(crate) bitmaps: [Option<Bitmap8>; 3],
    pub(crate) zmaps: [Option<ZMap>; 3],
    pub(crate) needs_sort: bool,
}

#[derive(Debug, Clone)]
pub struct DatFile {
    pub app_name: String,
    pub description: String,
    pub groups: Vec<GroupData>,
}

#[derive(Debug)]
struct DatFileHeader {
    file_size: u32,
    app_name: String,
    description: String,
    number_of_groups: u16,
    size_of_body: u32,
    unknown: u16,
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
    let _ = header.file_size;
    let _ = header.size_of_body;
    let _ = header.unknown;

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

    let width = width.max(0) as usize;
    let height = height.max(0) as usize;
    let stride = stride.max(0) as usize;

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

fn read_dat_header(file: &mut File) -> Result<DatFileHeader, String> {
    let mut signature = [0u8; 21];
    file.read_exact(&mut signature)
        .map_err(|error| format!("failed to read DAT signature: {error}"))?;
    if &signature[..DAT_SIGNATURE.len()] != DAT_SIGNATURE {
        return Err("DAT signature mismatch".to_string());
    }

    let app_name = read_c_string(file, 50)?;
    let description = read_c_string(file, 100)?;
    let file_size =
        read_u32(file).map_err(|error| format!("failed to read DAT file size: {error}"))?;
    let number_of_groups =
        read_u16(file).map_err(|error| format!("failed to read DAT group count: {error}"))?;
    let size_of_body =
        read_u32(file).map_err(|error| format!("failed to read DAT body size: {error}"))?;
    let unknown = read_u16(file)
        .map_err(|error| format!("failed to read DAT unknown header value: {error}"))?;

    Ok(DatFileHeader {
        file_size,
        app_name,
        description,
        number_of_groups,
        size_of_body,
        unknown,
    })
}

fn read_bitmap8_header(file: &mut File) -> Result<Dat8BitBmpHeader, String> {
    Ok(Dat8BitBmpHeader {
        resolution: read_u8(file)
            .map_err(|error| format!("failed to read bitmap8 resolution: {error}"))?,
        width: read_i16(file).map_err(|error| format!("failed to read bitmap8 width: {error}"))?,
        height: read_i16(file)
            .map_err(|error| format!("failed to read bitmap8 height: {error}"))?,
        x_position: read_i16(file).map_err(|error| format!("failed to read bitmap8 x: {error}"))?,
        y_position: read_i16(file).map_err(|error| format!("failed to read bitmap8 y: {error}"))?,
        size: read_i32(file).map_err(|error| format!("failed to read bitmap8 size: {error}"))?,
        flags: read_u8(file).map_err(|error| format!("failed to read bitmap8 flags: {error}"))?,
    })
}

fn read_bitmap16_header(file: &mut File) -> Result<Dat16BitBmpHeader, String> {
    Ok(Dat16BitBmpHeader {
        width: read_i16(file).map_err(|error| format!("failed to read zMap width: {error}"))?,
        height: read_i16(file).map_err(|error| format!("failed to read zMap height: {error}"))?,
        stride: read_i16(file).map_err(|error| format!("failed to read zMap stride: {error}"))?,
        _unknown0: read_i32(file)
            .map_err(|error| format!("failed to read zMap header: {error}"))?,
        _unknown1_0: read_i16(file)
            .map_err(|error| format!("failed to read zMap header: {error}"))?,
        _unknown1_1: read_i16(file)
            .map_err(|error| format!("failed to read zMap header: {error}"))?,
    })
}

fn read_exact_vec(file: &mut File, len: usize) -> std::io::Result<Vec<u8>> {
    let mut buffer = vec![0u8; len];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

fn read_c_string(file: &mut File, len: usize) -> Result<String, String> {
    let bytes =
        read_exact_vec(file, len).map_err(|error| format!("failed to read string: {error}"))?;
    Ok(clean_c_string(&bytes))
}

fn clean_c_string(bytes: &[u8]) -> String {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).trim().to_string()
}

fn read_u8(file: &mut File) -> std::io::Result<u8> {
    let mut buffer = [0u8; 1];
    file.read_exact(&mut buffer)?;
    Ok(buffer[0])
}

fn read_u16(file: &mut File) -> std::io::Result<u16> {
    let mut buffer = [0u8; 2];
    file.read_exact(&mut buffer)?;
    Ok(u16::from_le_bytes(buffer))
}

fn read_u32(file: &mut File) -> std::io::Result<u32> {
    let mut buffer = [0u8; 4];
    file.read_exact(&mut buffer)?;
    Ok(u32::from_le_bytes(buffer))
}

fn read_i16(file: &mut File) -> std::io::Result<i16> {
    let mut buffer = [0u8; 2];
    file.read_exact(&mut buffer)?;
    Ok(i16::from_le_bytes(buffer))
}

fn read_i32(file: &mut File) -> std::io::Result<i32> {
    let mut buffer = [0u8; 4];
    file.read_exact(&mut buffer)?;
    Ok(i32::from_le_bytes(buffer))
}
