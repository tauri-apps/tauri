//! Asset handling during codegen.

use std::path::PathBuf;
use thiserror::Error;

mod disk;
mod embedded;

pub use self::disk::DiskAssets;
pub use self::embedded::EmbeddedAssets;

/// All errors that can happen during codegen asset generation.
#[derive(Debug, Error)]
pub enum Error {
  #[error("failed to read asset at {path} because {error}")]
  AssetRead {
    path: PathBuf,
    error: std::io::Error,
  },

  #[error("failed to write asset from {path} to Vec<u8> because {error}")]
  AssetWrite {
    path: PathBuf,
    error: std::io::Error,
  },

  #[error("invalid prefix {prefix} used while including path {path}")]
  PrefixInvalid { prefix: PathBuf, path: PathBuf },

  #[error("failed to walk directory {path} because {error}")]
  Walkdir {
    path: PathBuf,
    error: walkdir::Error,
  },

  #[error("unable to canonicalize path {path} because {error}")]
  Canonicalize {
    path: PathBuf,
    error: std::io::Error,
  },

  #[error("OUT_DIR env var is not set, do you have a build script?")]
  OutDir,
}
