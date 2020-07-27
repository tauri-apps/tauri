//! Assets handled by Tauri during compile time and runtime.

use flate2::read::{GzDecoder, GzEncoder};
pub use phf;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

/// Type of compression applied to an asset
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AssetCompression {
  /// No compression applied
  None,

  /// Compressed with (gzip)[https://crates.io/crates/flate2]
  Gzip,
}

/// How the embedded asset should be fetched from `Assets`
pub enum AssetFetch {
  /// Do not modify the compression
  Identity,

  /// Ensure asset is decompressed
  Decompress,

  /// Ensure asset is compressed
  Compress,
}

/// Runtime access to the included files
pub struct Assets {
  inner: phf::Map<&'static str, (AssetCompression, &'static [u8])>,
}

impl Assets {
  /// Create `Assets` container from `phf::Map`
  pub const fn new(map: phf::Map<&'static str, (AssetCompression, &'static [u8])>) -> Self {
    Self { inner: map }
  }

  /// Format a key used to identify a file embedded in `Assets`.
  ///
  /// Output should use unix path separators and have a root directory to mimic
  /// server urls.
  pub fn format_key(path: impl Into<PathBuf>) -> String {
    let path = path.into();

    // add in root to mimic how it is used from a server url
    let path = if path.has_root() {
      path
    } else {
      Path::new(&Component::RootDir).join(path)
    };

    if cfg!(windows) {
      let mut buf = String::new();
      for component in path.components() {
        match component {
          Component::RootDir => buf.push('/'),
          Component::CurDir => buf.push_str("./"),
          Component::ParentDir => buf.push_str("../"),
          Component::Prefix(prefix) => buf.push_str(&prefix.as_os_str().to_string_lossy()),
          Component::Normal(s) => {
            buf.push_str(&s.to_string_lossy());
            buf.push('/')
          }
        }
      }

      // remove the last slash
      if buf != "/" {
        buf.pop();
      }

      buf
    } else {
      path.to_string_lossy().to_string()
    }
  }

  /// Get embedded asset, automatically handling compression.
  pub fn get(
    &self,
    path: impl Into<PathBuf>,
    fetch: AssetFetch,
  ) -> Option<(Box<dyn Read>, AssetCompression)> {
    use self::{AssetCompression::*, AssetFetch::*};

    let key = Self::format_key(path);
    let &(compression, content) = self.inner.get(&*key)?;
    Some(match (compression, fetch) {
      // content is already in compression format expected
      (_, Identity) | (None, Decompress) | (Gzip, Compress) => (Box::new(content), compression),

      // content is uncompressed, but fetched with compression
      (None, Compress) => {
        let compressor = GzEncoder::new(content, flate2::Compression::new(6));
        (Box::new(compressor), Gzip)
      }

      // content is compressed, but fetched with decompression
      (Gzip, Decompress) => {
        let decompressor = GzDecoder::new(content);
        (Box::new(decompressor), None)
      }
    })
  }
}
