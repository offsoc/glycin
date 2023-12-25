use super::{Frame, ImageInfo, MemoryFormat, SharedMemory};
use crate::FrameDetails;

#[derive(Default, Clone, Debug)]
pub struct Handler {
    pub format_name: Option<String>,
    pub default_bit_depth: Option<u8>,
    pub supports_two_alpha_modes: bool,
    pub supports_two_grayscale_modes: bool,
}

impl Handler {
    pub fn format_name(mut self, format_name: impl ToString) -> Self {
        self.format_name = Some(format_name.to_string()).into();

        self
    }

    pub fn default_bit_depth(mut self, default_bit_depth: u8) -> Self {
        self.default_bit_depth = Some(default_bit_depth).into();

        self
    }

    pub fn supports_two_alpha_modes(mut self, supports_two_alpha_modes: bool) -> Self {
        self.supports_two_alpha_modes = supports_two_alpha_modes.into();

        self
    }

    pub fn supports_two_grayscale_modes(mut self, supports_two_grayscale_modes: bool) -> Self {
        self.supports_two_grayscale_modes = supports_two_grayscale_modes.into();

        self
    }

    pub fn info<'a, T: image::ImageDecoder<'a>>(&self, decoder: &mut T) -> ImageInfo {
        let (width, height) = decoder.dimensions();
        let mut info = ImageInfo::new(width, height);
        info.details.format_name = self.format_name.clone();

        info
    }

    pub fn frame<'a, T: image::ImageDecoder<'a>>(
        &self,
        mut decoder: T,
    ) -> Result<Frame, image::ImageError> {
        let details = self.frame_details(&mut decoder);
        let color_type = decoder.color_type();

        let memory_format = MemoryFormat::from(color_type);
        let (width, height) = decoder.dimensions();

        let mut memory = SharedMemory::new(decoder.total_bytes());
        decoder.read_image(&mut memory)?;
        let texture = memory.into_texture();

        let mut frame = Frame::new(width, height, memory_format, texture);
        frame.details = details;

        Ok(frame)
    }

    pub fn frame_details<'a, T: image::ImageDecoder<'a>>(&self, decoder: &mut T) -> FrameDetails {
        let mut details = FrameDetails::default();

        details.iccp = decoder.icc_profile();

        if let Some((alpha_channel, grayscale, bits)) =
            channel_details(decoder.original_color_type())
        {
            if self.default_bit_depth != Some(bits) {
                details.bit_depth = Some(bits);
            }
            if self.supports_two_alpha_modes {
                details.alpha_channel = Some(alpha_channel);
            }
            if self.supports_two_grayscale_modes {
                details.grayscale = Some(grayscale);
            }
        }

        details
    }
}

impl ImageInfo {
    pub fn from_decoder<'a, T: image::ImageDecoder<'a>>(
        decoder: &mut T,
        _format_name: impl ToString,
    ) -> Self {
        let (width, height) = decoder.dimensions();

        Self::new(width, height)
    }
}

impl From<image::ColorType> for MemoryFormat {
    fn from(color_type: image::ColorType) -> Self {
        match color_type {
            image::ColorType::L8 => Self::G8,
            image::ColorType::La8 => Self::G8a8,
            image::ColorType::Rgb8 => Self::R8g8b8,
            image::ColorType::Rgba8 => Self::R8g8b8a8,
            image::ColorType::L16 => Self::G16,
            image::ColorType::La16 => Self::G16a16,
            image::ColorType::Rgb16 => Self::R16g16b16,
            image::ColorType::Rgba16 => Self::R16g16b16a16,
            image::ColorType::Rgb32F => Self::R32g32b32Float,
            image::ColorType::Rgba32F => Self::R32g32b32Float,
            _ => unimplemented!(),
        }
    }
}

fn channel_details(color_type: image::ExtendedColorType) -> Option<(bool, bool, u8)> {
    Some(match color_type {
        image::ExtendedColorType::A8 => (true, false, 8),
        image::ExtendedColorType::L1 => (false, true, 1),
        image::ExtendedColorType::La1 => (true, true, 1),
        image::ExtendedColorType::Rgb1 => (false, false, 1),
        image::ExtendedColorType::Rgba1 => (true, false, 1),
        image::ExtendedColorType::L2 => (false, true, 2),
        image::ExtendedColorType::La2 => (true, true, 2),
        image::ExtendedColorType::Rgb2 => (false, false, 2),
        image::ExtendedColorType::Rgba2 => (true, false, 2),
        image::ExtendedColorType::L4 => (false, true, 4),
        image::ExtendedColorType::La4 => (true, true, 4),
        image::ExtendedColorType::Rgb4 => (false, false, 4),
        image::ExtendedColorType::Rgba4 => (true, false, 4),
        image::ExtendedColorType::L8 => (false, true, 8),
        image::ExtendedColorType::La8 => (true, true, 8),
        image::ExtendedColorType::Rgb8 => (false, false, 8),
        image::ExtendedColorType::Rgba8 => (true, false, 8),
        image::ExtendedColorType::L16 => (false, true, 16),
        image::ExtendedColorType::La16 => (true, true, 16),
        image::ExtendedColorType::Rgb16 => (false, false, 16),
        image::ExtendedColorType::Rgba16 => (true, false, 16),
        image::ExtendedColorType::Bgr8 => (false, false, 8),
        image::ExtendedColorType::Bgra8 => (true, false, 8),
        image::ExtendedColorType::Rgb32F => (false, false, 32),
        image::ExtendedColorType::Rgba32F => (true, false, 32),
        image::ExtendedColorType::Unknown(bits) => (false, false, bits),
        _ => return None,
    })
}
