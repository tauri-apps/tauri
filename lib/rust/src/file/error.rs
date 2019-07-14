use std;
use zip::result::ZipError;

#[derive(Debug)]
pub enum Error {
    Extract(String),
    Io(std::io::Error),
    Zip(ZipError)
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        use Error::*;
        match *self {
            Extract(ref s) => write!(f, "ExtractError: {}", s),
            Io(ref e) => write!(f, "IoError: {}", e),
            Zip(ref e) => write!(f, "ZipError: {}", e),
        }
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        "File Error"
    }

    fn cause(&self) -> Option<&std::error::Error> {
        use Error::*;
        Some(match *self {
            Io(ref e) => e,
            _ => return None,
        })
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e)
    }
}

impl From<ZipError> for Error {
    fn from(e: ZipError) -> Self {
        Error::Zip(e)
    }
}
