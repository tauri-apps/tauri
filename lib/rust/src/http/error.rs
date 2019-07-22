use reqwest;
use serde_json;
use std;

#[derive(Debug)]
pub enum Error {
  Download(String),
  Json(serde_json::Error),
  Reqwest(reqwest::Error),
  Io(std::io::Error),
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Error::*;
    match *self {
      Download(ref s) => write!(f, "DownloadError: {}", s),
      Json(ref e) => write!(f, "JsonError: {}", e),
      Reqwest(ref e) => write!(f, "ReqwestError: {}", e),
      Io(ref e) => write!(f, "IoError: {}", e),
    }
  }
}

impl std::error::Error for Error {
  fn description(&self) -> &str {
    "Http Error"
  }

  fn cause(&self) -> Option<&dyn std::error::Error> {
    use Error::*;
    Some(match *self {
      Json(ref e) => e,
      Reqwest(ref e) => e,
      Io(ref e) => e,
      _ => return None,
    })
  }
}

impl From<serde_json::Error> for Error {
  fn from(e: serde_json::Error) -> Self {
    Error::Json(e)
  }
}

impl From<reqwest::Error> for Error {
  fn from(e: reqwest::Error) -> Self {
    Error::Reqwest(e)
  }
}

impl From<std::io::Error> for Error {
  fn from(e: std::io::Error) -> Self {
    Error::Io(e)
  }
}
