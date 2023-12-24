#![allow(clippy::large_enum_variant)]

use glycin_utils::*;

use std::io::Cursor;
use std::io::Read;
use std::sync::Mutex;

use jxl_oxide::{JxlImage, PixelFormat};

fn main() {
    Communication::spawn(ImgDecoder::default());
}

#[derive(Default)]
pub struct ImgDecoder {
    pub decoder: Mutex<Option<JxlImage>>,
}

impl Decoder for ImgDecoder {
    fn init(
        &self,
        stream: UnixStream,
        _mime_type: String,
        _details: InitializationDetails,
    ) -> Result<ImageInfo, DecoderError> {
        let image = JxlImage::from_reader(stream).unwrap();

        let header = image.image_header();

        let mut image_info = ImageInfo::new(header.size.width, header.size.height);
        image_info.details.format_name = Some(String::from("JPEG XL")).into();

        *self.decoder.lock().unwrap() = Some(image);

        Ok(image_info)
    }

    fn decode_frame(&self, _frame_request: FrameRequest) -> Result<Frame, DecoderError> {
        let Some(image) = std::mem::take(&mut *self.decoder.lock().unwrap()) else {
            return Err(DecoderError::InternalDecoderError);
        };

        let buffer = image
            .render_frame(0)
            .map_err(|x| DecoderError::DecodingError(x.to_string()))?
            .image();

        // Buffer with channel size u16 = 2 bytes
        let mut memory = SharedMemory::new(buffer.buf().len().try_u64()? * 2);

        let u16_buffer: Vec<u16> = buffer
            .buf()
            .iter()
            .map(|x| (x * u16::MAX as f32) as u16)
            .collect();

        Cursor::new(safe_transmute::transmute_to_bytes(&u16_buffer))
            .read_exact(&mut memory)
            .unwrap();
        let texture = memory.into_texture();
        let memory_format = pixel_to_memory_format(image.pixel_format());

        let mut frame = Frame::new(
            buffer.width().try_u32()?,
            buffer.height().try_u32()?,
            memory_format,
            texture,
        );
        frame.details.iccp = Some(image.rendered_icc()).into();

        Ok(frame)
    }
}

fn pixel_to_memory_format(format: PixelFormat) -> MemoryFormat {
    match format {
        PixelFormat::Gray => MemoryFormat::G16,
        PixelFormat::Graya => MemoryFormat::G16a16,
        PixelFormat::Rgb => MemoryFormat::R16g16b16,
        PixelFormat::Rgba => MemoryFormat::R16g16b16a16,
        PixelFormat::Cmyk => MemoryFormat::R16g16b16,
        PixelFormat::Cmyka => MemoryFormat::R16g16b16a16,
    }
}
