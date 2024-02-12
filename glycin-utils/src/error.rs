use anyhow::Context;
use gettextrs::gettext;

#[derive(zbus::DBusError, Debug, Clone)]
#[zbus(prefix = "org.gnome.glycin.Error")]
pub enum RemoteError {
    #[zbus(error)]
    ZBus(zbus::Error),
    LoadingError(String),
    InternalLoaderError,
    UnsupportedImageFormat(String),
    ConversionTooLargerError,
}

impl From<LoaderError> for RemoteError {
    fn from(err: LoaderError) -> Self {
        match err {
            LoaderError::LoadingError(msg) => Self::LoadingError(msg),
            LoaderError::InternalLoaderError => Self::InternalLoaderError,
            LoaderError::UnsupportedImageFormat(msg) => Self::UnsupportedImageFormat(msg),
            LoaderError::ConversionTooLargerError => Self::ConversionTooLargerError,
        }
    }
}

#[derive(Debug)]
pub enum LoaderError {
    LoadingError(String),
    InternalLoaderError,
    UnsupportedImageFormat(String),
    ConversionTooLargerError,
}

impl std::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            Self::LoadingError(err) => write!(f, "{err}"),
            Self::InternalLoaderError => {
                write!(f, "{}", gettext("Internal error while interpreting image"))
            }
            Self::UnsupportedImageFormat(msg) => {
                write!(f, "{} {msg}", gettext("Unsupported image format: "))
            }
            err @ Self::ConversionTooLargerError => err.fmt(f),
        }
    }
}

impl std::error::Error for LoaderError {}

impl From<anyhow::Error> for LoaderError {
    fn from(err: anyhow::Error) -> Self {
        eprintln!("Decoding error: {err:?}");
        Self::LoadingError(format!("{err}: {}", err.root_cause()))
    }
}

impl From<DimensionTooLargerError> for LoaderError {
    fn from(err: DimensionTooLargerError) -> Self {
        eprintln!("Decoding error: {err:?}");
        Self::ConversionTooLargerError
    }
}

pub trait GenericContexts<T> {
    fn context_failed(self) -> anyhow::Result<T>;
    fn context_internal(self) -> Result<T, LoaderError>;
    fn context_unsupported(self, msg: String) -> Result<T, LoaderError>;
}

impl<T, E> GenericContexts<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn context_failed(self) -> anyhow::Result<T> {
        self.with_context(|| gettext("Failed to decode image"))
    }

    fn context_internal(self) -> Result<T, LoaderError> {
        self.map_err(|_| LoaderError::InternalLoaderError)
    }

    fn context_unsupported(self, msg: String) -> Result<T, LoaderError> {
        self.map_err(|_| LoaderError::UnsupportedImageFormat(msg))
    }
}

impl<T> GenericContexts<T> for Option<T> {
    fn context_failed(self) -> anyhow::Result<T> {
        self.with_context(|| gettext("Failed to decode image"))
    }

    fn context_internal(self) -> Result<T, LoaderError> {
        self.ok_or(LoaderError::InternalLoaderError)
    }

    fn context_unsupported(self, msg: String) -> Result<T, LoaderError> {
        self.ok_or(LoaderError::UnsupportedImageFormat(msg))
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
