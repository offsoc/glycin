#![allow(clippy::large_enum_variant)]

use std::io::{Cursor, Read};
use std::sync::Mutex;

use glycin_utils::*;
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

        let mut memory = SharedMemory::new(buffer.buf().len().try_u64()?);

        let u8_buffer: Vec<u8> = buffer
            .buf()
            .iter()
            .map(|x| (x * u8::MAX as f32) as u8)
            .collect();

        Cursor::new(&u8_buffer).read_exact(&mut memory).unwrap();
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
        PixelFormat::Gray => MemoryFormat::G8,
        PixelFormat::Graya => MemoryFormat::G8a8,
        PixelFormat::Rgb => MemoryFormat::R8g8b8,
        PixelFormat::Rgba => MemoryFormat::R8g8b8a8,
        PixelFormat::Cmyk => MemoryFormat::R8g8b8,
        PixelFormat::Cmyka => MemoryFormat::R8g8b8a8,
    }
}
