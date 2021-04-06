//! Assets handled by Tauri during compile time and runtime.

pub use phf;
use std::path::PathBuf;
use std::{
  borrow::Cow,
  path::{Component, Path},
};

/// Represent an asset file path in a normalized way.
///
/// The following rules are enforced and added if needed:
/// * Unix path component separators
/// * Has a root directory
/// * No trailing slash - directories are not included in assets
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct AssetKey(String);

impl From<AssetKey> for String {
  fn from(key: AssetKey) -> Self {
    key.0
  }
}

impl AsRef<str> for AssetKey {
  fn as_ref(&self) -> &str {
    &self.0
  }
}

impl<P: AsRef<Path>> From<P> for AssetKey {
  fn from(path: P) -> Self {
    // TODO: change this to utilize `Cow` to prevent allocating an intermediate `PathBuf` when not necessary
    let path = path.as_ref().to_owned();

    // add in root to mimic how it is used from a server url
    let path = if path.has_root() {
      path
    } else {
      Path::new(&Component::RootDir).join(path)
    };

    let buf = if cfg!(windows) {
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
    };

    AssetKey(buf)
  }
}

/// Represents a container of file assets that are retrievable during runtime.
pub trait Assets: Send + Sync + 'static {
  /// Get the content of the passed [`AssetKey`].
  fn get<Key: Into<AssetKey>>(&self, key: Key) -> Option<Cow<'_, [u8]>>;
}

/// [`Assets`] implementation that reads files from dist_dir disk at runtime
///
/// Note: this should only be used for development as that directory of assets is not likely to
/// exists on the end user's machine unless you put it there for every install.
///
/// See [`EmbeddedAssets`] for the recommended way for including release assets.
pub struct DiskAssets(PathBuf);

impl DiskAssets {
  /// The passed path should be a canonical path so that long paths are supported under Windows.
  pub fn from_dist_dir(path: impl Into<PathBuf>) -> Self {
    Self(path.into())
  }
}

impl Assets for DiskAssets {
  fn get<Key: Into<AssetKey>>(&self, key: Key) -> Option<Cow<'_, [u8]>> {
    // strip the absolute path root off the key
    let key = key.into();
    let key = &key.as_ref()[1..];

    std::fs::read(self.0.join(key)).map(Cow::Owned).ok()
  }
}

/// [`Assets`] implementation that only contains compile-time compressed and embedded assets.
pub struct EmbeddedAssets(phf::Map<&'static str, &'static [u8]>);

impl EmbeddedAssets {
  /// Wrap a [zstd] compressed [`phf::Map`].
  ///
  /// [zstd]: https://facebook.github.io/zstd/
  pub const fn from_zstd(map: phf::Map<&'static str, &'static [u8]>) -> Self {
    Self(map)
  }
}

impl Assets for EmbeddedAssets {
  fn get<Key: Into<AssetKey>>(&self, key: Key) -> Option<Cow<'_, [u8]>> {
    self
      .0
      .get(key.into().as_ref())
      .copied()
      .map(zstd::decode_all)
      .and_then(Result::ok)
      .map(Cow::Owned)
  }
}
