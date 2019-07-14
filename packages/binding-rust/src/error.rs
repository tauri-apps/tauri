use std::{
    error,
    ffi::NulError,
    fmt::{self, Debug, Display},
};

pub trait CustomError: Display + Debug + Send + Sync + 'static {}

impl<T: Display + Debug + Send + Sync + 'static> CustomError for T {}

/// A WebView error.
#[derive(Debug)]
pub enum Error {
    /// While attempting to build a WebView instance, a required field was not initialized.
    UninitializedField(&'static str),
    /// An error occurred while initializing a WebView instance.
    Initialization,
    /// A nul-byte was found in a provided string.
    NulByte(NulError),
    /// An error occurred while evaluating JavaScript in a WebView instance.
    JsEvaluation,
    /// An error occurred while injecting CSS into a WebView instance.
    CssInjection,
    /// Failure to dispatch a closure to a WebView instance via a handle, likely because the
    /// WebView was dropped.
    Dispatch,
    /// An user-specified error occurred. For use inside invoke and dispatch closures.
    Custom(Box<CustomError>),
}

impl Error {
    /// Creates a custom error from a `T: Display + Debug + Send + Sync + 'static`.
    pub fn custom<E: CustomError>(error: E) -> Error {
        Error::Custom(Box::new(error))
    }
}

impl error::Error for Error {
    fn cause(&self) -> Option<&error::Error> {
        match self {
            Error::NulByte(cause) => Some(cause),
            _ => None,
        }
    }

    #[cfg(feature = "V1_30")]
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::NulByte(ref cause) => Some(cause),
            _ => None,
        }
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::UninitializedField(field) => {
                write!(f, "Required field uninitialized: {}.", field)
            }
            Error::Initialization => write!(f, "Webview failed to initialize."),
            Error::NulByte(cause) => write!(f, "{}", cause),
            Error::JsEvaluation => write!(f, "Failed to evaluate JavaScript."),
            Error::CssInjection => write!(f, "Failed to inject CSS."),
            Error::Dispatch => write!(
                f,
                "Closure could not be dispatched. WebView was likely dropped."
            ),
            Error::Custom(e) => write!(f, "Error: {}", e),
        }
    }
}

/// A WebView result.
pub type WVResult<T = ()> = Result<T, Error>;

impl From<NulError> for Error {
    fn from(e: NulError) -> Error {
        Error::NulByte(e)
    }
}
