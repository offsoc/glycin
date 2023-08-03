use gio;
use glycin_utils::anyhow::Context;
use glycin_utils::*;
use std::io::Cursor;
use std::io::Read;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Mutex;

/// Current librsvg limit on maximum dimensions. See
/// <https://gitlab.gnome.org/GNOME/librsvg/-/issues/938>
pub const RSVG_MAX_SIZE: u32 = 32_767;

fn main() {
    Communication::spawn(ImgDecoder::default());
}

#[derive(Default)]
pub struct ImgDecoder {
    thread: Mutex<Option<ImgDecoderDetails>>,
}

pub struct ImgDecoderDetails {
    frame_recv: Receiver<Result<Frame, DecoderError>>,
    instr_send: Sender<Instruction>,
    image_info: ImageInfo,
}

pub struct Instruction {
    total_size: (u32, u32),
    area: cairo::Rectangle,
}

pub fn thread(
    stream: UnixStream,
    info_send: Sender<Result<ImageInfo, DecoderError>>,
    frame_send: Sender<Result<Frame, DecoderError>>,
    instr_recv: Receiver<Instruction>,
) {
    let input_stream = unsafe { gio::UnixInputStream::take_fd(stream) };

    // TODO: Base url
    let handle = rsvg::Loader::new()
        .read_stream(&input_stream, gio::File::NONE, gio::Cancellable::NONE)
        .unwrap();
    let renderer = rsvg::CairoRenderer::new(&handle);

    let (original_width, original_height) = svg_dimensions(&renderer);

    let image_info = ImageInfo::new(original_width, original_height, String::from("SVG"));
    info_send.send(Ok(image_info)).unwrap();

    // TODO: Detailled dimensions info
    /*
                let intrisic_dimensions = renderer.intrinsic_dimensions();

    tiles.set_original_dimensions_full(
                    (original_width, original_height),
                    ImageDimensionDetails::Svg((intrisic_dimensions.width, intrisic_dimensions.height)),
                );
                */

    while let Ok(instr) = instr_recv.recv() {
        dbg!("INSTRUCTION");
        let (total_width, total_height) = instr.total_size;

        // librsvg does not currently support larger images
        if total_height > RSVG_MAX_SIZE || total_width > RSVG_MAX_SIZE {
            continue;
        }

        let frame = render(&renderer, instr);

        frame_send.send(frame).unwrap();
    }
}

pub fn render(renderer: &rsvg::CairoRenderer, instr: Instruction) -> Result<Frame, DecoderError> {
    let area = instr.area;
    let (total_width, total_height) = instr.total_size;

    let surface = cairo::ImageSurface::create(
        cairo::Format::ARgb32,
        area.width() as i32,
        area.height() as i32,
    )
    .unwrap();

    let context = cairo::Context::new(&surface).context("Failed to create new cairo context")?;

    renderer
        .render_document(
            &context,
            &cairo::Rectangle::new(
                -area.x() as f64,
                -area.y() as f64,
                total_width as f64,
                total_height as f64,
            ),
        )
        .context("Failed to render image")?;

    drop(context);

    let width = surface.width();
    let height = surface.height();
    let stride = surface.stride() as usize;

    let data = surface
        .take_data()
        .context("Cairo surface already taken")?
        .to_vec();

    let mut memory = SharedMemory::new(data.len().try_u64()?);

    Cursor::new(data).read_exact(&mut memory).unwrap();
    let texture = memory.into_texture();

    let mut frame = Frame::new(
        width.try_u32()?,
        height.try_u32()?,
        memory_format(),
        texture,
    );

    frame.stride = stride.try_u32()?;

    Ok(frame)
}

impl Decoder for ImgDecoder {
    fn init(&self, stream: UnixStream, _mime_type: String) -> Result<ImageInfo, DecoderError> {
        let (info_send, info_recv) = channel();
        let (frame_send, frame_recv) = channel();
        let (instr_send, instr_recv) = channel();

        std::thread::spawn(move || thread(stream, info_send, frame_send, instr_recv));
        let image_info = info_recv.recv().unwrap()?;

        *self.thread.lock().unwrap() = Some(ImgDecoderDetails {
            frame_recv,
            instr_send,
            image_info: image_info.clone(),
        });

        Ok(image_info)
    }

    fn decode_frame(&self, frame_request: FrameRequest) -> Result<Frame, DecoderError> {
        let lock = self.thread.lock().unwrap();
        let thread = lock.as_ref().context_internal()?;

        let image_info = &thread.image_info;
        let width = image_info.width;
        let height = image_info.height;

        let total_size = frame_request.scale.unwrap_or((width, height));
        let area = if let Some(clip) = *frame_request.clip {
            cairo::Rectangle::new(clip.0.into(), clip.1.into(), clip.2.into(), clip.3.into())
        } else {
            cairo::Rectangle::new(0., 0., width.into(), height.into())
        };

        let instr = Instruction { total_size, area };

        thread.instr_send.send(instr).unwrap();

        thread.frame_recv.recv().unwrap()
    }
}

pub fn svg_dimensions(renderer: &rsvg::CairoRenderer) -> (u32, u32) {
    if let Some((width, height)) = renderer.intrinsic_size_in_pixels() {
        (width.ceil() as u32, height.ceil() as u32)
    } else {
        match renderer.intrinsic_dimensions() {
            rsvg::IntrinsicDimensions {
                width:
                    rsvg::Length {
                        length: width,
                        unit: rsvg::LengthUnit::Percent,
                    },
                height:
                    rsvg::Length {
                        length: height,
                        unit: rsvg::LengthUnit::Percent,
                    },
                vbox: Some(vbox),
            } => (
                (width * vbox.width()).ceil() as u32,
                (height * vbox.height()).ceil() as u32,
            ),
            dimensions => {
                eprintln!("Failed to parse SVG dimensions: {dimensions:?}");
                (300, 300)
            }
        }
    }
}

const fn memory_format() -> MemoryFormat {
    #[cfg(target_endian = "little")]
    {
        MemoryFormat::B8g8r8a8
    }

    #[cfg(target_endian = "big")]
    {
        MemoryFormat::A8r8g8b8
    }
}
