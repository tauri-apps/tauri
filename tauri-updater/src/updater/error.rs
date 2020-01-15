use crate::http;
use reqwest;
use std;
use tauri_api;
// use tauri_api::file;
// use tauri_api::version;
use zip::result::ZipError;

#[derive(Debug)]
pub enum Error {
  Updater(String),
  Release(String),
  Network(String),
  Config(String),
  Io(std::io::Error),
  Zip(ZipError),
  API(tauri_api::Error),
  // File(file::Error),
  // Version(version::Error),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Error::*;
    match *self {
      Updater(ref s) => write!(f, "UpdaterError: {}", s),
      Release(ref s) => write!(f, "ReleaseError: {}", s),
      Network(ref s) => write!(f, "NetworkError: {}", s),
      Config(ref s) => write!(f, "ConfigError: {}", s),
      Io(ref e) => write!(f, "IoError: {}", e),
      Zip(ref e) => write!(f, "ZipError: {}", e),
      API(ref e) => write!(f, "APIError: {}", e),
      // File(ref e) => write!(f, "FileError: {}", e),
      // Version(ref e) => write!(f, "VersionError: {}", e),
    }
  }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    "Updater Error"
  }

  fn cause(&self) -> Option<&dyn std::error::Error> {
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

// impl From<file::Error> for Error {
//   fn from(e: file::Error) -> Self {
//     Error::File(e)
//   }
// }

impl From<http::Error> for Error {
  fn from(e: http::Error) -> Self {
    Error::Network(e.to_string())
  }
}

impl From<reqwest::Error> for Error {
  fn from(e: reqwest::Error) -> Self {
    Error::Network(e.to_string())
  }
}

impl From<tauri_api::Error> for Error {
  fn from(e: tauri_api::Error) -> Self {
    Error::API(e)
  }
}

// impl From<version::Error> for Error {
//   fn from(e: version::Error) -> Self {
//     Error::Version(e)
//   }
// }
