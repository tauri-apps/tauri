//! Assets handled by Tauri during compile time and runtime.

use flate2::read::GzDecoder;
pub use phf;
use phf::Map;
use std::{
  io::Read,
  path::{Component, Path},
};

/// How the embedded asset should be fetched from [`Assets`].
pub enum AssetFetch {
  /// Fetch an asset without compression
  Decompressed,

  /// Fetch an asset with compression
  Compressed,
}

/// Format a path as a key ready to be used in an [`Assets`] container.
///
/// Output should use unix path separators and have a root directory to mimic server urls. This is
/// already provided with a cross-platform implementation as a convenience.
pub fn format_key(path: impl AsRef<Path>) -> String {
  // TODO: change this to utilize `Cow` to prevent allocating an intermediate `PathBuf` when not necessary
  let path = path.as_ref().to_owned();

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

/// Represents a container of Compressible assets that can be fetched during runtime.
pub trait Assets {
  /// Get asset, automatically handling gzip compression.
  fn get<P: AsRef<Path>>(&self, path: P, fetch: AssetFetch) -> Option<Box<dyn Read>>;
}

/// [`Assets`] implementation with entirely static embedded assets.
pub struct EmbeddedAssets(phf::Map<&'static str, &'static [u8]>);

impl EmbeddedAssets {
  /// Wrap [`phf::Map`] into an [`EmbeddedAssets`].
  pub const fn new(map: Map<&'static str, &'static [u8]>) -> Self {
    Self(map)
  }
}

impl Assets for EmbeddedAssets {
  fn get<P: AsRef<Path>>(&self, path: P, fetch: AssetFetch) -> Option<Box<dyn Read>> {
    let key = format_key(path);
    self.0.get(&*key).map(|&bytes| match fetch {
      AssetFetch::Compressed => Box::new(bytes) as Box<dyn Read>,
      AssetFetch::Decompressed => {
        let decompressor = GzDecoder::new(bytes);
        Box::new(decompressor) as Box<dyn Read>
      }
    })
  }
}
