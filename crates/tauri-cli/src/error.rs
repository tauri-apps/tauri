// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Display, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}: {1}")]
  Context(String, Box<dyn std::error::Error + Send + Sync + 'static>),
  #[error("{0}")]
  GenericError(String),
  #[error("failed to bundle project {0}")]
  Bundler(#[from] Box<tauri_bundler::Error>),
  #[error("{context} {path}: {error}")]
  Fs {
    context: &'static str,
    path: PathBuf,
    error: std::io::Error,
  },
  #[error("failed to run command {command}: {error}")]
  CommandFailed {
    command: String,
    error: std::io::Error,
  },
  #[cfg(target_os = "macos")]
  #[error(transparent)]
  MacosSign(#[from] Box<tauri_macos_sign::Error>),
}

/// Convenient type alias of Result type.
pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T> {
  // Required methods
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static;
  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C;
}

impl<T, E: std::error::Error + Send + Sync + 'static> Context<T> for std::result::Result<T, E> {
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self.map_err(|e| Error::Context(context.to_string(), Box::new(e)))
  }

  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self.map_err(|e| Error::Context(f().to_string(), Box::new(e)))
  }
}

impl<T> Context<T> for Option<T> {
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self.ok_or_else(|| Error::GenericError(context.to_string()))
  }

  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self.ok_or_else(|| Error::GenericError(f().to_string()))
  }
}

pub trait ErrorExt<T> {
  fn fs_context(self, context: &'static str, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> ErrorExt<T> for std::result::Result<T, std::io::Error> {
  fn fs_context(self, context: &'static str, path: impl Into<PathBuf>) -> Result<T> {
    self.map_err(|error| Error::Fs {
      context,
      path: path.into(),
      error,
    })
  }
}

macro_rules! bail {
   ($msg:literal $(,)?) => {
      return Err(crate::Error::GenericError($msg.into()))
   };
    ($err:expr $(,)?) => {
       return Err(crate::Error::GenericError($err))
    };
   ($fmt:expr, $($arg:tt)*) => {
     return Err(crate::Error::GenericError(format!($fmt, $($arg)*)))
   };
}

pub(crate) use bail;
