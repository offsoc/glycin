use std::process::ExitStatus;
use std::sync::Arc;

use futures_channel::oneshot;
use gio::glib;
use glycin_utils::{DimensionTooLargerError, RemoteError};
use libseccomp::error::SeccompError;

use crate::MimeType;

#[derive(Debug, Clone, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
    #[error("Remote error: {0}")]
    RemoteError(#[from] RemoteError),
    #[error("GLib error: {0}")]
    GLibError(#[from] glib::Error),
    #[error("IO error: {err} {info}")]
    StdIoError {
        err: Arc<std::io::Error>,
        info: String,
    },
    #[error("D-Bus error: {0}")]
    DbusError(#[from] zbus::Error),
    #[error("Internal communication was unexpectedly canceled")]
    InternalCommunicationCanceled,
    #[error("Unknown image format: {0}")]
    UnknownImageFormat(MimeType),
    #[error("Loader process exited early with status '{}'. {cmd}", .status.code().unwrap_or_default())]
    PrematureExit { status: ExitStatus, cmd: String },
    #[error("Conversion too large")]
    ConversionTooLargerError,
    #[error("Could not spawn `{cmd}`: {err}")]
    SpawnError {
        cmd: String,
        err: Arc<std::io::Error>,
    },
    #[error("Texture is only {texture_size} but was announced differently: {frame}")]
    TextureTooSmall { texture_size: usize, frame: String },
    #[error("Stride is smaller than possible: {0}")]
    StrideTooSmall(String),
    #[error("Width or height is zero: {0}")]
    WidgthOrHeightZero(String),
    #[error("Memfd: {0}")]
    MemFd(Arc<memfd::Error>),
    #[error("Seccomp: {0}")]
    Seccomp(Arc<SeccompError>),
    #[error("ICC profile: {0}")]
    IccProfile(#[from] lcms2::Error),
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

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::StdIoError {
            err: Arc::new(err),
            info: String::new(),
        }
    }
}

impl From<memfd::Error> for Error {
    fn from(err: memfd::Error) -> Self {
        Self::MemFd(Arc::new(err))
    }
}

impl From<SeccompError> for Error {
    fn from(err: SeccompError) -> Self {
        Self::Seccomp(Arc::new(err))
    }
}

impl From<oneshot::Canceled> for Error {
    fn from(_err: oneshot::Canceled) -> Self {
        Self::InternalCommunicationCanceled
    }
}

impl From<DimensionTooLargerError> for Error {
    fn from(_err: DimensionTooLargerError) -> Self {
        Self::ConversionTooLargerError
    }
}
