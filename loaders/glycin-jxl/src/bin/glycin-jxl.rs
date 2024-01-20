#![allow(clippy::large_enum_variant)]

use std::io::{Cursor, Read, Write};
use std::mem::MaybeUninit;
use std::sync::Mutex;

use glycin_utils::*;
use jpegxl_rs::image::ToDynamic;
use jpegxl_sys::*;

fn main() {
    Communication::spawn(ImgDecoder::default());
}

#[derive(Default)]
pub struct ImgDecoder {
    pub decoder: Mutex<Option<Vec<u8>>>,
}

impl Decoder for ImgDecoder {
    fn init(
        &self,
        mut stream: UnixStream,
        _mime_type: String,
        _details: InitializationDetails,
    ) -> Result<ImageInfo, DecoderError> {
        let mut data = Vec::new();
        stream.read_to_end(&mut data).context_failed()?;
        let info = basic_info(&data).context_failed()?;

        let mut image_info = ImageInfo::new(info.xsize, info.ysize);
        image_info.details.format_name = Some(String::from("JPEG XL"));

        *self.decoder.lock().unwrap() = Some(data);

        Ok(image_info)
    }

    fn decode_frame(&self, _frame_request: FrameRequest) -> Result<Frame, DecoderError> {
        let Some(data) = std::mem::take(&mut *self.decoder.lock().unwrap()) else {
            return Err(DecoderError::InternalDecoderError);
        };

        let decoder = jpegxl_rs::decode::decoder_builder()
            .build()
            .context_failed()?;

        let image = decoder
            .decode_to_image(&data)
            .context_failed()?
            .context_failed()?;

        let memory_format = MemoryFormat::from(image.color());
        let (alpha_channel, grayscale, bits) =
            image_rs::channel_details(image.color().into()).context_internal()?;
        let width = image.width();
        let height = image.height();

        let bytes = image.into_bytes();
        let mut memory = SharedMemory::new(bytes.len() as u64);

        Cursor::new(memory.as_mut())
            .write_all(&bytes)
            .context_internal()?;
        let texture = memory.into_texture();

        let mut frame = Frame::new(width, height, memory_format, texture).context_failed()?;

        if bits != 8 {
            frame.details.bit_depth = Some(bits);
        }

        if alpha_channel {
            frame.details.alpha_channel = Some(true);
        }

        if grayscale {
            frame.details.grayscale = Some(true);
        }

        Ok(frame)
    }
}

fn basic_info(data: &[u8]) -> Option<JxlBasicInfo> {
    unsafe {
        let decoder = JxlDecoderCreate(std::ptr::null());

        JxlDecoderSubscribeEvents(decoder, JxlDecoderStatus::BasicInfo as i32);
        JxlDecoderSetInput(decoder, data.as_ptr(), data.len());
        JxlDecoderCloseInput(decoder);

        let mut basic_info = None;

        let status = JxlDecoderProcessInput(decoder);
        if status == JxlDecoderStatus::BasicInfo {
            let mut info = MaybeUninit::uninit();
            if JxlDecoderGetBasicInfo(decoder, info.as_mut_ptr()) == JxlDecoderStatus::Success {
                basic_info = Some(info.assume_init());
            }
        }

        basic_info
    }
}
