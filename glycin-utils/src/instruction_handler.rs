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

        let instruction_handler = DecodingInstruction {
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

struct DecodingInstruction {
    decoder: Mutex<Box<dyn Decoder>>,
}

#[zbus::dbus_interface(name = "org.gnome.glycin.DecodingInstruction")]
impl DecodingInstruction {
    async fn init(&self, message: InitializationRequest) -> Result<ImageInfo, RemoteError> {
        let fd = message.fd.into_raw_fd();
        let stream = unsafe { UnixStream::from_raw_fd(fd) };

        let image_info = self
            .decoder
            .lock()
            .or(Err(RemoteError::InternalDecoderError))?
            .init(stream, message.mime_type, message.details)?;

        Ok(image_info)
    }

    async fn decode_frame(&self, frame_request: FrameRequest) -> Result<Frame, RemoteError> {
        self.decoder
            .lock()
            .or(Err(RemoteError::InternalDecoderError))?
            .decode_frame(frame_request)
            .map_err(Into::into)
    }
}
