pub use anyhow;

use std::os::fd::AsRawFd;
use std::os::fd::{FromRawFd, IntoRawFd};
pub use std::os::unix::net::UnixStream;
use std::sync::Mutex;

use crate::dbus::*;
use crate::error::*;

pub struct Communication {
    _dbus_connection: zbus::Connection,
}

impl Communication {
    pub fn spawn(decoder: impl Decoder + 'static) {
        async_std::task::block_on(async move {
            let _connection = Communication::new(decoder).await;
            std::future::pending::<()>().await;
        })
    }

    pub async fn new(decoder: impl Decoder + 'static) -> Self {
        let unix_stream = unsafe { UnixStream::from_raw_fd(std::io::stdin().as_raw_fd()) };

        let instruction_handler = Decoding {
            decoder: Mutex::new(Box::new(decoder)),
        };
        let dbus_connection = zbus::ConnectionBuilder::unix_stream(unix_stream)
            .p2p()
            .auth_mechanisms(&[zbus::AuthMechanism::Anonymous])
            .serve_at("/org/gnome/glycin", instruction_handler)
            .expect("Failed to setup instruction handler")
            .build()
            .await
            .expect("Failed to create private DBus connection");

        Communication {
            _dbus_connection: dbus_connection,
        }
    }
}

pub trait Decoder: Send {
    fn init(
        &self,
        stream: UnixStream,
        mime_type: String,
        details: InitializationDetails,
    ) -> Result<ImageInfo, DecoderError>;
    fn decode_frame(&self, frame_request: FrameRequest) -> Result<Frame, DecoderError>;
}

pub struct Decoding {
    pub decoder: Mutex<Box<dyn Decoder>>,
}

#[zbus::dbus_interface(name = "org.gnome.glycin.Loader")]
impl Decoding {
    async fn init(&self, init_request: InitRequest) -> Result<ImageInfo, RemoteError> {
        let fd = init_request.fd.into_raw_fd();
        let stream = unsafe { UnixStream::from_raw_fd(fd) };

        let image_info = self
            .decoder
            .lock()
            .or(Err(RemoteError::InternalDecoderError))?
            .init(stream, init_request.mime_type, init_request.details)?;

        Ok(image_info)
    }

    async fn frame(&self, frame_request: FrameRequest) -> Result<Frame, RemoteError> {
        self.decoder
            .lock()
            .or(Err(RemoteError::InternalDecoderError))?
            .decode_frame(frame_request)
            .map_err(Into::into)
    }
}
