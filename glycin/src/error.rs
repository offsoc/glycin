use std::process::ExitStatus;
use std::sync::Arc;

use futures_channel::oneshot;
use gdk::glib;
use glycin_utils::{DimensionTooLargerError, RemoteError};
use libseccomp::error::SeccompError;

pub type StdResult<T, E> = std::result::Result<T, E>;

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
    Seccomp(Arc<SeccompError>),
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
            Self::Seccomp(err) => write!(f, "Seccomp: {err}"),
        }
    }
}

impl std::error::Error for Error {}
