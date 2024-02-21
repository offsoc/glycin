#[derive(zbus::DBusError, Debug, Clone)]
#[zbus(prefix = "org.gnome.glycin.Error")]
#[non_exhaustive]
pub enum RemoteError {
    #[zbus(error)]
    ZBus(zbus::Error),
    LoadingError(String),
    InternalLoaderError(String),
    UnsupportedImageFormat(String),
    ConversionTooLargerError,
    LoadingErrors(String),
}

type Location = std::panic::Location<'static>;

impl From<LoaderError> for RemoteError {
    fn from(err: LoaderError) -> Self {
        match err {
            LoaderError::LoadingError(msg) => Self::LoadingError(msg),
            LoaderError::LoadingErrors { err, location } => {
                Self::LoadingErrors(format!("{location}: {err}"))
            }
            LoaderError::InternalLoaderError { err, location } => {
                Self::InternalLoaderError(format!("{location}: {err}"))
            }
            LoaderError::UnsupportedImageFormat(msg) => Self::UnsupportedImageFormat(msg),
            LoaderError::ConversionTooLargerError => Self::ConversionTooLargerError,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum LoaderError {
    #[error("{0}")]
    LoadingError(String),
    #[error("{err}")]
    LoadingErrors { err: String, location: Location },
    #[error("Internal error while interpreting image")]
    InternalLoaderError { err: String, location: Location },
    #[error("Unsupported image format: {0}")]
    UnsupportedImageFormat(String),
    #[error("Dimension too large for system")]
    ConversionTooLargerError,
}

impl From<DimensionTooLargerError> for LoaderError {
    fn from(err: DimensionTooLargerError) -> Self {
        eprintln!("Decoding error: {err:?}");
        Self::ConversionTooLargerError
    }
}

pub trait GenericContexts<T> {
    fn loading_error(self, location: impl FnOnce() -> Location) -> Result<T, LoaderError>;
    fn internal_error(self, location: impl FnOnce() -> Location) -> Result<T, LoaderError>;
}

impl<T, E> GenericContexts<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn loading_error(self, location: impl FnOnce() -> Location) -> Result<T, LoaderError> {
        self.map_err(|err| LoaderError::LoadingErrors {
            err: err.to_string(),
            location: location(),
        })
    }

    fn internal_error(self, location: impl FnOnce() -> Location) -> Result<T, LoaderError> {
        self.map_err(|err| LoaderError::InternalLoaderError {
            err: err.to_string(),
            location: location(),
        })
    }
}

impl<T> GenericContexts<T> for Option<T> {
    fn loading_error(self, location: impl FnOnce() -> Location) -> Result<T, LoaderError> {
        self.ok_or_else(|| LoaderError::LoadingErrors {
            err: String::from("None"),
            location: location(),
        })
    }

    fn internal_error(self, location: impl FnOnce() -> Location) -> Result<T, LoaderError> {
        self.ok_or_else(|| LoaderError::LoadingErrors {
            err: String::from("None"),
            location: location(),
        })
    }
}

#[derive(Debug)]
pub struct DimensionTooLargerError;

impl std::fmt::Display for DimensionTooLargerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_str("Dimension too large for system")
    }
}

impl std::error::Error for DimensionTooLargerError {}

#[macro_export]
macro_rules! error_context {
    ($x:expr) => {{
        $x.loading_error(|| *std::panic::Location::caller())
    }};
}

#[macro_export]
macro_rules! internal_error_context {
    ($x:expr) => {{
        $x.internal_error(|| *std::panic::Location::caller())
    }};
}
