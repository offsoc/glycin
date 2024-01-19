use anyhow::Context;
use gettextrs::gettext;

#[derive(zbus::DBusError, Debug, Clone)]
#[dbus_error(prefix = "org.gnome.glycin.Error")]
pub enum RemoteError {
    #[dbus_error(zbus_error)]
    ZBus(zbus::Error),
    DecodingError(String),
    InternalDecoderError,
    UnsupportedImageFormat(String),
    ConversionTooLargerError,
}

impl From<DecoderError> for RemoteError {
    fn from(err: DecoderError) -> Self {
        match err {
            DecoderError::DecodingError(msg) => Self::DecodingError(msg),
            DecoderError::InternalDecoderError => Self::InternalDecoderError,
            DecoderError::UnsupportedImageFormat(msg) => Self::UnsupportedImageFormat(msg),
            DecoderError::ConversionTooLargerError => Self::ConversionTooLargerError,
        }
    }
}

#[derive(Debug)]
pub enum DecoderError {
    DecodingError(String),
    InternalDecoderError,
    UnsupportedImageFormat(String),
    ConversionTooLargerError,
}

impl std::fmt::Display for DecoderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::DecodingError(err) => write!(f, "{err}"),
            Self::InternalDecoderError => {
                write!(f, "{}", gettext("Internal error while interpreting image"))
            }
            Self::UnsupportedImageFormat(msg) => {
                write!(f, "{} {msg}", gettext("Unsupported image format: "))
            }
            err @ Self::ConversionTooLargerError => err.fmt(f),
        }
    }
}

impl std::error::Error for DecoderError {}

impl From<anyhow::Error> for DecoderError {
    fn from(err: anyhow::Error) -> Self {
        eprintln!("Decoding error: {err:?}");
        Self::DecodingError(format!("{err}: {}", err.root_cause()))
    }
}

impl From<DimensionTooLargerError> for DecoderError {
    fn from(err: DimensionTooLargerError) -> Self {
        eprintln!("Decoding error: {err:?}");
        Self::ConversionTooLargerError
    }
}

pub trait GenericContexts<T> {
    fn context_failed(self) -> anyhow::Result<T>;
    fn context_internal(self) -> Result<T, DecoderError>;
    fn context_unsupported(self, msg: String) -> Result<T, DecoderError>;
}

impl<T, E> GenericContexts<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context_failed(self) -> anyhow::Result<T> {
        self.with_context(|| gettext("Failed to decode image"))
    }

    fn context_internal(self) -> Result<T, DecoderError> {
        self.map_err(|_| DecoderError::InternalDecoderError)
    }

    fn context_unsupported(self, msg: String) -> Result<T, DecoderError> {
        self.map_err(|_| DecoderError::UnsupportedImageFormat(msg))
    }
}

impl<T> GenericContexts<T> for Option<T> {
    fn context_failed(self) -> anyhow::Result<T> {
        self.with_context(|| gettext("Failed to decode image"))
    }

    fn context_internal(self) -> Result<T, DecoderError> {
        self.ok_or(DecoderError::InternalDecoderError)
    }

    fn context_unsupported(self, msg: String) -> Result<T, DecoderError> {
        self.ok_or(DecoderError::UnsupportedImageFormat(msg))
    }
}

#[derive(Debug)]
pub struct DimensionTooLargerError;

impl std::fmt::Display for DimensionTooLargerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str(&gettext("Dimension too large for system"))
    }
}

impl std::error::Error for DimensionTooLargerError {}
