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
}

type Location = std::panic::Location<'static>;

impl From<LoaderError> for RemoteError {
    fn from(err: LoaderError) -> Self {
        match err {
            err @ LoaderError::LoadingError { .. } => Self::LoadingError(err.to_string()),
            err @ LoaderError::InternalLoaderError { .. } => {
                Self::InternalLoaderError(err.to_string())
            }
            LoaderError::UnsupportedImageFormat(msg) => Self::UnsupportedImageFormat(msg),
            LoaderError::ConversionTooLargerError => Self::ConversionTooLargerError,
        }
    }
}

#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum LoaderError {
    #[error("{location}: {err}")]
    LoadingError { err: String, location: Location },
    #[error("inernal error: {location}: {err}")]
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
    fn loading_error(self) -> Result<T, LoaderError>;
    fn internal_error(self) -> Result<T, LoaderError>;
}

impl<T, E> GenericContexts<T> for Result<T, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    #[track_caller]
    fn loading_error(self) -> Result<T, LoaderError> {
        match self {
            Ok(x) => Ok(x),
            Err(err) => Err(LoaderError::LoadingError {
                err: err.to_string(),
                location: *Location::caller(),
            }),
        }
    }

    #[track_caller]
    fn internal_error(self) -> Result<T, LoaderError> {
        match self {
            Ok(x) => Ok(x),
            Err(err) => Err(LoaderError::InternalLoaderError {
                err: err.to_string(),
                location: *Location::caller(),
            }),
        }
    }
}

impl<T> GenericContexts<T> for Option<T> {
    #[track_caller]
    fn loading_error(self) -> Result<T, LoaderError> {
        match self {
            Some(x) => Ok(x),
            None => Err(LoaderError::LoadingError {
                err: String::from("None"),
                location: *Location::caller(),
            }),
        }
    }

    #[track_caller]
    fn internal_error(self) -> Result<T, LoaderError> {
        match self {
            Some(x) => Ok(x),
            None => Err(LoaderError::InternalLoaderError {
                err: String::from("None"),
                location: *Location::caller(),
            }),
        }
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
