//! Internal DBus API

use std::os::fd::{AsRawFd, FromRawFd};
use std::os::unix::net::UnixStream;
use std::process::ExitStatus;
use std::sync::Arc;

use async_global_executor::{block_on, spawn_blocking};
use futures_channel::oneshot;
use futures_util::{future, FutureExt};
use gdk::prelude::*;
use gio::glib;
use glycin_utils::{
    DimensionTooLargerError, Frame, FrameRequest, ImageInfo, InitRequest, InitializationDetails,
    MemoryFormat, RemoteError, SafeConversion, SafeMath, Texture,
};
use nix::sys::signal;
use zbus::zvariant;

use crate::api::{self, SandboxMechanism};
use crate::config;
use crate::sandbox::Sandbox;

#[derive(Clone, Debug)]
pub struct DecoderProcess<'a> {
    _dbus_connection: zbus::Connection,
    decoding_instruction: LoaderProxy<'a>,
    mime_type: String,
}

impl<'a> DecoderProcess<'a> {
    pub async fn new(
        mime_type: &config::MimeType,
        config: &config::Config,
        sandbox_mechanism: SandboxMechanism,
        file: &gio::File,
        cancellable: &gio::Cancellable,
    ) -> Result<DecoderProcess<'a>, Error> {
        let loader_config = config.get(mime_type)?;

        // UnixStream which facilitates the D-Bus connection. The stream is passed as
        // stdin to loader binaries.
        let (unix_stream, loader_stdin) = std::os::unix::net::UnixStream::pair()?;
        unix_stream
            .set_nonblocking(true)
            .expect("Couldn't set nonblocking");
        loader_stdin
            .set_nonblocking(true)
            .expect("Couldn't set nonblocking");

        let decoder_bin = loader_config.exec.clone();
        let mut sandbox = Sandbox::new(sandbox_mechanism, decoder_bin, loader_stdin);
        // Mount dir that contains the file as read only for formats like SVG
        if loader_config.expose_base_dir {
            if let Some(base_dir) = file.parent().and_then(|x| x.path()) {
                sandbox.add_ro_bind(base_dir);
            }
        }
        let (mut subprocess, cmd_debug) = sandbox.spawn().await?;

        #[cfg(feature = "tokio")]
        let unix_stream = tokio::net::UnixStream::from_std(unix_stream)?;

        let guid = zbus::Guid::generate();
        let dbus_result = zbus::ConnectionBuilder::unix_stream(unix_stream)
            .p2p()
            .server(&guid)
            .auth_mechanisms(&[zbus::AuthMechanism::Anonymous])
            .build()
            .shared();

        let subprocess_id = nix::unistd::Pid::from_raw(subprocess.id().try_into().unwrap());

        futures_util::select! {
            _result = dbus_result.clone().fuse() => Ok(()),
            _result = cancellable.future().fuse() => {
                let _result = signal::kill(subprocess_id, signal::Signal::SIGKILL);
                Err(glib::Error::from(gio::Cancelled).into())
            },
            return_status = spawn_blocking(move || subprocess.wait()).fuse() => match return_status {
                Ok(status) => Err(Error::PrematureExit(status, cmd_debug)),
                Err(err) => Err(Error::StdIoError(err.into(), cmd_debug)),
            }
        }?;

        cancellable.connect_cancelled_local(move |_| {
            let _result = signal::kill(subprocess_id, signal::Signal::SIGKILL);
        });

        let dbus_connection = dbus_result.await?;

        let decoding_instruction = LoaderProxy::new(&dbus_connection)
            .await
            .expect("Failed to create decoding instruction proxy");

        Ok(Self {
            _dbus_connection: dbus_connection,
            decoding_instruction,
            mime_type: mime_type.to_string(),
        })
    }

    pub async fn init(
        &self,
        gfile_worker: GFileWorker,
        base_dir: Option<std::path::PathBuf>,
    ) -> Result<ImageInfo, Error> {
        let (remote_reader, writer) = std::os::unix::net::UnixStream::pair()?;

        gfile_worker.write_to(writer)?;

        let fd = unsafe { zvariant::OwnedFd::from_raw_fd(remote_reader.as_raw_fd()) };
        std::mem::forget(remote_reader);

        let mime_type = self.mime_type.clone();

        let mut details = InitializationDetails::default();
        details.base_dir = base_dir;

        let image_info = self
            .decoding_instruction
            .init(InitRequest {
                fd,
                mime_type,
                details,
            })
            .shared();

        let reader_error = gfile_worker.error();
        futures_util::pin_mut!(reader_error);

        futures_util::select! {
            _result = image_info.clone().fuse() => Ok(()),
            result = reader_error.fuse() => result,
        }?;

        image_info.await.map_err(Into::into)
    }

    pub async fn decode_frame(&self, frame_request: FrameRequest) -> Result<api::Frame, Error> {
        let mut frame = self.decoding_instruction.frame(frame_request).await?;

        let Texture::MemFd(fd) = &frame.texture;
        let raw_fd = fd.as_raw_fd();
        let borrowed_fd = unsafe { std::os::fd::BorrowedFd::borrow_raw(raw_fd) };
        let mut original_mmap = unsafe { memmap::MmapMut::map_mut(raw_fd) }?;

        if original_mmap.len() < frame.n_bytes()? {
            return Err(Error::TextureTooSmall {
                texture_size: original_mmap.len(),
                frame: format!("{:?}", frame),
            });
        }

        if frame.stride < frame.width.smul(frame.memory_format.n_bytes().u32())? {
            return Err(Error::StrideTooSmall(format!("{:?}", frame)));
        }

        if let Some(icc_profile) = frame.details.iccp.clone() {
            // Align stride with pixel size if necessary
            let icc_mmap = if frame.stride.srem(frame.memory_format.n_bytes().u32())? != 0 {
                let width = frame
                    .width
                    .try_usize()?
                    .smul(frame.memory_format.n_bytes().usize())?;
                let stride = frame.stride.try_usize()?;
                let mut source = vec![0; width];
                for row in 1..frame.height.try_usize()? {
                    source.copy_from_slice(
                        &original_mmap[row.smul(stride)?..row.smul(stride)?.sadd(width)?],
                    );
                    original_mmap[row.smul(width)?..row.sadd(1)?.smul(width)?]
                        .copy_from_slice(&source);
                }
                frame.stride = width.try_u32()?;

                // This mmap would have the wrong size after ftruncate
                drop(original_mmap);

                nix::unistd::ftruncate(
                    borrowed_fd,
                    libc::off_t::try_from(frame.n_bytes()?).map_err(|_| DimensionTooLargerError)?,
                )
                .unwrap();

                // Need a new mmap with correct size
                unsafe { memmap::MmapMut::map_mut(raw_fd) }?
            } else {
                original_mmap
            };

            let memory_format = frame.memory_format;
            let icc_result: Result<(), anyhow::Error> = spawn_blocking(move || {
                crate::icc::apply_transformation(&icc_profile, memory_format, icc_mmap)
            })
            .await;

            if let Err(err) = icc_result {
                eprintln!("Failed to apply ICC profile: {err}");
            }
        } else {
            drop(original_mmap);
        }

        let mfd = memfd::Memfd::try_from_fd(raw_fd).unwrap();
        // 🦭
        mfd.add_seals(&[
            memfd::FileSeal::SealShrink,
            memfd::FileSeal::SealGrow,
            memfd::FileSeal::SealWrite,
            memfd::FileSeal::SealSeal,
        ])?;

        let bytes: glib::Bytes = unsafe {
            let mut error = std::ptr::null_mut();

            let mapped_file =
                glib::ffi::g_mapped_file_new_from_fd(raw_fd, glib::ffi::GFALSE, &mut error);

            if !error.is_null() {
                let err: glib::Error = glib::translate::from_glib_full(error);
                return Err(err.into());
            };

            let bytes =
                glib::translate::from_glib_full(glib::ffi::g_mapped_file_get_bytes(mapped_file));

            glib::ffi::g_mapped_file_unref(mapped_file);

            bytes
        };

        let texture = gdk::MemoryTexture::new(
            frame.width.try_i32()?,
            frame.height.try_i32()?,
            gdk_memory_format(frame.memory_format),
            &bytes,
            frame.stride.try_usize()?,
        );

        Ok(api::Frame {
            texture: texture.upcast(),
            delay: frame.delay.into(),
            details: frame.details,
        })
    }
}

use std::io::Write;
const BUF_SIZE: usize = u16::MAX as usize;

#[zbus::dbus_proxy(
    interface = "org.gnome.glycin.Loader",
    default_path = "/org/gnome/glycin"
)]
trait Loader {
    async fn init(&self, init_request: InitRequest) -> Result<ImageInfo, RemoteError>;
    async fn frame(&self, frame_request: FrameRequest) -> Result<Frame, RemoteError>;
}

const fn gdk_memory_format(format: MemoryFormat) -> gdk::MemoryFormat {
    match format {
        MemoryFormat::B8g8r8a8Premultiplied => gdk::MemoryFormat::B8g8r8a8Premultiplied,
        MemoryFormat::A8r8g8b8Premultiplied => gdk::MemoryFormat::A8r8g8b8Premultiplied,
        MemoryFormat::R8g8b8a8Premultiplied => gdk::MemoryFormat::R8g8b8a8Premultiplied,
        MemoryFormat::B8g8r8a8 => gdk::MemoryFormat::B8g8r8a8,
        MemoryFormat::A8r8g8b8 => gdk::MemoryFormat::A8r8g8b8,
        MemoryFormat::R8g8b8a8 => gdk::MemoryFormat::R8g8b8a8,
        MemoryFormat::A8b8g8r8 => gdk::MemoryFormat::A8b8g8r8,
        MemoryFormat::R8g8b8 => gdk::MemoryFormat::R8g8b8,
        MemoryFormat::B8g8r8 => gdk::MemoryFormat::B8g8r8,
        MemoryFormat::R16g16b16 => gdk::MemoryFormat::R16g16b16,
        MemoryFormat::R16g16b16a16Premultiplied => gdk::MemoryFormat::R16g16b16a16Premultiplied,
        MemoryFormat::R16g16b16a16 => gdk::MemoryFormat::R16g16b16a16,
        MemoryFormat::R16g16b16Float => gdk::MemoryFormat::R16g16b16Float,
        MemoryFormat::R16g16b16a16Float => gdk::MemoryFormat::R16g16b16a16Float,
        MemoryFormat::R32g32b32Float => gdk::MemoryFormat::R32g32b32Float,
        MemoryFormat::R32g32b32a32FloatPremultiplied => {
            gdk::MemoryFormat::R32g32b32a32FloatPremultiplied
        }
        MemoryFormat::R32g32b32a32Float => gdk::MemoryFormat::R32g32b32a32Float,
        MemoryFormat::G8a8Premultiplied => gdk::MemoryFormat::G8a8Premultiplied,
        MemoryFormat::G8a8 => gdk::MemoryFormat::G8a8,
        MemoryFormat::G8 => gdk::MemoryFormat::G8,
        MemoryFormat::G16a16Premultiplied => gdk::MemoryFormat::G16a16Premultiplied,
        MemoryFormat::G16a16 => gdk::MemoryFormat::G16a16,
        MemoryFormat::G16 => gdk::MemoryFormat::G16,
    }
}

pub struct GFileWorker {
    file: gio::File,
    writer_send: Mutex<Option<oneshot::Sender<UnixStream>>>,
    first_bytes_recv: future::Shared<oneshot::Receiver<Arc<Vec<u8>>>>,
    error_recv: future::Shared<oneshot::Receiver<Result<(), Error>>>,
}
use std::sync::Mutex;
impl GFileWorker {
    pub fn spawn(file: gio::File, cancellable: gio::Cancellable) -> GFileWorker {
        let gfile = file.clone();

        let (error_send, error_recv) = oneshot::channel();
        let (first_bytes_send, first_bytes_recv) = oneshot::channel();
        let (writer_send, writer_recv) = oneshot::channel();

        spawn_blocking(move || {
            Self::handle_errors(error_send, move || {
                let reader = gfile.read(Some(&cancellable))?;
                let mut buf = vec![0; BUF_SIZE];

                let n = reader.read(&mut buf, Some(&cancellable))?;
                let first_bytes = Arc::new(buf[..n].to_vec());
                first_bytes_send
                    .send(first_bytes.clone())
                    .or(Err(Error::InternalCommunicationCanceled))?;

                let mut writer: UnixStream = block_on(writer_recv)?;

                writer.write_all(&first_bytes)?;
                drop(first_bytes);

                loop {
                    let n = reader.read(&mut buf, Some(&cancellable))?;
                    if n == 0 {
                        break;
                    }
                    writer.write_all(&buf[..n])?;
                }

                Ok(())
            })
        })
        .detach();

        GFileWorker {
            file,
            writer_send: Mutex::new(Some(writer_send)),
            first_bytes_recv: first_bytes_recv.shared(),
            error_recv: error_recv.shared(),
        }
    }

    fn handle_errors(
        error_send: oneshot::Sender<Result<(), Error>>,
        f: impl FnOnce() -> Result<(), Error>,
    ) {
        let result = f();
        let _result = error_send.send(result);
    }

    pub fn write_to(&self, stream: UnixStream) -> Result<(), Error> {
        let sender = std::mem::take(&mut *self.writer_send.lock().unwrap());

        sender
            // TODO: this fails if write_to is called a second time
            .unwrap()
            .send(stream)
            .or(Err(Error::InternalCommunicationCanceled))
    }

    pub fn file(&self) -> &gio::File {
        &self.file
    }

    pub async fn error(&self) -> Result<(), Error> {
        match self.error_recv.clone().await {
            Ok(result) => result,
            Err(_) => Ok(()),
        }
    }

    pub async fn head(&self) -> Result<Arc<Vec<u8>>, Error> {
        futures_util::select!(
            err = self.error_recv.clone() => err?,
            _bytes = self.first_bytes_recv.clone() => Ok(()),
        )?;

        match self.first_bytes_recv.clone().await {
            Err(_) => self.error_recv.clone().await?.map(|_| Default::default()),
            Ok(bytes) => Ok(bytes),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Error {
    RemoteError(RemoteError),
    GLibError(glib::Error),
    StdIoError(Arc<std::io::Error>, String),
    InternalCommunicationCanceled,
    UnknownImageFormat(String),
    PrematureExit(ExitStatus, String),
    ConversionTooLargerError,
    SpawnError(String, Arc<std::io::Error>),
    TextureTooSmall { texture_size: usize, frame: String },
    StrideTooSmall(String),
    MemFd(Arc<memfd::Error>),
}

impl Error {
    pub fn unsupported_format(&self) -> Option<String> {
        match self {
            Self::UnknownImageFormat(mime_type) => Some(mime_type.clone()),
            Self::RemoteError(RemoteError::UnsupportedImageFormat(msg)) => Some(msg.clone()),
            _ => None,
        }
    }
}

impl From<RemoteError> for Error {
    fn from(err: RemoteError) -> Self {
        Self::RemoteError(err)
    }
}

impl From<glib::Error> for Error {
    fn from(err: glib::Error) -> Self {
        Self::GLibError(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::StdIoError(Arc::new(err), String::new())
    }
}

impl From<memfd::Error> for Error {
    fn from(err: memfd::Error) -> Self {
        Self::MemFd(Arc::new(err))
    }
}

impl From<oneshot::Canceled> for Error {
    fn from(_err: oneshot::Canceled) -> Self {
        Self::InternalCommunicationCanceled
    }
}

impl From<zbus::Error> for Error {
    fn from(err: zbus::Error) -> Self {
        Self::RemoteError(RemoteError::ZBus(err))
    }
}

impl From<DimensionTooLargerError> for Error {
    fn from(_err: DimensionTooLargerError) -> Self {
        Self::ConversionTooLargerError
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> StdResult<(), std::fmt::Error> {
        match self {
            Self::RemoteError(err) => write!(f, "{err}"),
            Self::GLibError(err) => write!(f, "{err}"),
            Self::StdIoError(err, info) => write!(f, "{err} {info}"),
            Self::InternalCommunicationCanceled => {
                write!(f, "Internal communication was unexpectedly canceled")
            }
            Self::UnknownImageFormat(mime_type) => {
                write!(f, "Unknown image format: {mime_type}")
            }
            Self::PrematureExit(status, command) => {
                write!(
                    f,
                    "Loader process exited early with status '{}'. {command}",
                    status.code().unwrap_or_default()
                )
            }
            err @ Self::ConversionTooLargerError => err.fmt(f),
            Self::SpawnError(cmd, err) => write!(f, "Could not spawn `{cmd}`: {err}"),
            Self::TextureTooSmall {
                texture_size,
                frame,
            } => write!(
                f,
                "Texture is only {texture_size} but was announced differently: {frame}"
            ),
            Self::StrideTooSmall(frame) => write!(f, "Stride is smaller than possible: {frame}"),
            Self::MemFd(err) => write!(f, "Memfd: {err}"),
        }
    }
}

impl std::error::Error for Error {}

pub type StdResult<T, E> = std::result::Result<T, E>;
