use serde::{Deserialize, Serialize};
use zbus::zvariant;
use zbus::zvariant::{Optional, Type};

use std::time::Duration;

#[derive(Deserialize, Serialize, Type, Debug)]
pub struct DecodingRequest {
    /// Source from which the loader reads the image data
    pub fd: zvariant::OwnedFd,
    pub details: DecodingDetails,
}

#[derive(Deserialize, Serialize, Type, Debug)]
pub struct DecodingDetails {
    pub mime_type: String,
    pub base_dir: Optional<std::path::PathBuf>,
}

#[derive(Deserialize, Serialize, Type, Debug, Clone, Default)]
pub struct FrameRequest {
    pub scale: Optional<(u32, u32)>,
    /// Instruction to only decode part of the image
    pub clip: Optional<(u32, u32, u32, u32)>,
}

/// Various image metadata
#[derive(Deserialize, Serialize, Type, Debug, Clone)]
pub struct ImageInfo {
    pub width: u32,
    pub height: u32,
    pub format_name: String,
    pub exif: Optional<Vec<u8>>,
    pub xmp: Optional<Vec<u8>>,
    pub transformations_applied: bool,
    pub dimensions_text: Optional<String>,
    pub dimensions_inch: Optional<(f64, f64)>,
}

impl ImageInfo {
    pub fn new(width: u32, height: u32, format_name: String) -> Self {
        Self {
            width,
            height,
            format_name,
            exif: None.into(),
            xmp: None.into(),
            transformations_applied: false,
            dimensions_text: None.into(),
            dimensions_inch: None.into(),
        }
    }
}

#[derive(Deserialize, Serialize, Type, Debug)]
pub struct Frame {
    pub width: u32,
    pub height: u32,
    pub stride: u32,
    pub memory_format: MemoryFormat,
    pub texture: Texture,
    pub iccp: Optional<Vec<u8>>,
    pub cicp: Optional<Vec<u8>>,
    pub delay: Optional<Duration>,
}

impl Frame {
    pub fn new(width: u32, height: u32, memory_format: MemoryFormat, texture: Texture) -> Self {
        let stride = memory_format.n_bytes().u32() * width;

        Self {
            width,
            height,
            stride,
            memory_format,
            texture,
            iccp: None.into(),
            cicp: None.into(),
            delay: None.into(),
        }
    }
}

#[derive(Deserialize, Serialize, Type, Debug)]
pub enum Texture {
    MemFd(zvariant::OwnedFd),
}

#[derive(Deserialize, Serialize, Type, Debug, Clone, Copy)]
pub enum MemoryFormat {
    B8g8r8a8Premultiplied,
    A8r8g8b8Premultiplied,
    R8g8b8a8Premultiplied,
    B8g8r8a8,
    A8r8g8b8,
    R8g8b8a8,
    A8b8g8r8,
    R8g8b8,
    B8g8r8,
    R16g16b16,
    R16g16b16a16Premultiplied,
    R16g16b16a16,
    R16g16b16Float,
    R16g16b16a16Float,
    R32g32b32Float,
    R32g32b32a32FloatPremultiplied,
    R32g32b32a32Float,
    G8a8Premultiplied,
    G8a8,
    G8,
    G16a16Premultiplied,
    G16a16,
    G16,
}

impl MemoryFormat {
    pub const fn n_bytes(self) -> MemoryFormatBytes {
        match self {
            MemoryFormat::B8g8r8a8Premultiplied => MemoryFormatBytes::B4,
            MemoryFormat::A8r8g8b8Premultiplied => MemoryFormatBytes::B4,
            MemoryFormat::R8g8b8a8Premultiplied => MemoryFormatBytes::B4,
            MemoryFormat::B8g8r8a8 => MemoryFormatBytes::B4,
            MemoryFormat::A8r8g8b8 => MemoryFormatBytes::B4,
            MemoryFormat::R8g8b8a8 => MemoryFormatBytes::B4,
            MemoryFormat::A8b8g8r8 => MemoryFormatBytes::B4,
            MemoryFormat::R8g8b8 => MemoryFormatBytes::B3,
            MemoryFormat::B8g8r8 => MemoryFormatBytes::B3,
            MemoryFormat::R16g16b16 => MemoryFormatBytes::B6,
            MemoryFormat::R16g16b16a16Premultiplied => MemoryFormatBytes::B8,
            MemoryFormat::R16g16b16a16 => MemoryFormatBytes::B8,
            MemoryFormat::R16g16b16Float => MemoryFormatBytes::B6,
            MemoryFormat::R16g16b16a16Float => MemoryFormatBytes::B8,
            MemoryFormat::R32g32b32Float => MemoryFormatBytes::B12,
            MemoryFormat::R32g32b32a32FloatPremultiplied => MemoryFormatBytes::B16,
            MemoryFormat::R32g32b32a32Float => MemoryFormatBytes::B16,
            MemoryFormat::G8a8Premultiplied => MemoryFormatBytes::B2,
            MemoryFormat::G8a8 => MemoryFormatBytes::B2,
            MemoryFormat::G8 => MemoryFormatBytes::B1,
            MemoryFormat::G16a16Premultiplied => MemoryFormatBytes::B4,
            MemoryFormat::G16a16 => MemoryFormatBytes::B4,
            MemoryFormat::G16 => MemoryFormatBytes::B2,
        }
    }

    pub const fn n_channels(self) -> u8 {
        match self {
            MemoryFormat::B8g8r8a8Premultiplied => 4,
            MemoryFormat::A8r8g8b8Premultiplied => 4,
            MemoryFormat::R8g8b8a8Premultiplied => 4,
            MemoryFormat::B8g8r8a8 => 4,
            MemoryFormat::A8r8g8b8 => 4,
            MemoryFormat::R8g8b8a8 => 4,
            MemoryFormat::A8b8g8r8 => 4,
            MemoryFormat::R8g8b8 => 3,
            MemoryFormat::B8g8r8 => 3,
            MemoryFormat::R16g16b16 => 3,
            MemoryFormat::R16g16b16a16Premultiplied => 4,
            MemoryFormat::R16g16b16a16 => 4,
            MemoryFormat::R16g16b16Float => 3,
            MemoryFormat::R16g16b16a16Float => 4,
            MemoryFormat::R32g32b32Float => 3,
            MemoryFormat::R32g32b32a32FloatPremultiplied => 4,
            MemoryFormat::R32g32b32a32Float => 4,
            MemoryFormat::G8a8Premultiplied => 2,
            MemoryFormat::G8a8 => 2,
            MemoryFormat::G8 => 1,
            MemoryFormat::G16a16Premultiplied => 2,
            MemoryFormat::G16a16 => 2,
            MemoryFormat::G16 => 1,
        }
    }
}

pub enum MemoryFormatBytes {
    B1 = 1,
    B2 = 2,
    B3 = 3,
    B4 = 4,
    B6 = 6,
    B8 = 8,
    B12 = 12,
    B16 = 16,
}

impl MemoryFormatBytes {
    pub fn u32(self) -> u32 {
        self as u32
    }

    pub fn u64(self) -> u64 {
        self as u64
    }

    pub fn usize(self) -> usize {
        self as usize
    }
}
