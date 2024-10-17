// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The Assets module allows you to read files that have been bundled by tauri
//! during both compile time and runtime.

#[doc(hidden)]
pub use phf;
use std::{
  borrow::Cow,
  path::{Component, Path},
};

/// Assets iterator.
pub type AssetsIter<'a> = dyn Iterator<Item = (Cow<'a, str>, Cow<'a, [u8]>)> + 'a;

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

/// A Content-Security-Policy hash value for a specific directive.
/// For more information see [the MDN page](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Content-Security-Policy#directives).
#[non_exhaustive]
#[derive(Debug, Clone, Copy)]
pub enum CspHash<'a> {
  /// The `script-src` directive.
  Script(&'a str),

  /// The `style-src` directive.
  Style(&'a str),
}

impl CspHash<'_> {
  /// The Content-Security-Policy directive this hash applies to.
  pub fn directive(&self) -> &'static str {
    match self {
      Self::Script(_) => "script-src",
      Self::Style(_) => "style-src",
    }
  }

  /// The value of the Content-Security-Policy hash.
  pub fn hash(&self) -> &str {
    match self {
      Self::Script(hash) => hash,
      Self::Style(hash) => hash,
    }
  }
}

/// [`Assets`] implementation that only contains compile-time compressed and embedded assets.
#[derive(Debug)]
pub struct EmbeddedAssets {
  assets: phf::Map<&'static str, &'static [u8]>,
  // Hashes that must be injected to the CSP of every HTML file.
  global_hashes: &'static [CspHash<'static>],
  // Hashes that are associated to the CSP of the HTML file identified by the map key (the HTML asset key).
  html_hashes: phf::Map<&'static str, &'static [CspHash<'static>]>,
}

impl EmbeddedAssets {
  /// Creates a new instance from the given asset map and script hash list.
  pub const fn new(
    map: phf::Map<&'static str, &'static [u8]>,
    global_hashes: &'static [CspHash<'static>],
    html_hashes: phf::Map<&'static str, &'static [CspHash<'static>]>,
  ) -> Self {
    Self {
      assets: map,
      global_hashes,
      html_hashes,
    }
  }

  /// Get an asset by key.
  #[cfg(feature = "compression")]
  pub fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    self
      .assets
      .get(key.as_ref())
      .map(|&(mut asdf)| {
        // with the exception of extremely small files, output should usually be
        // at least as large as the compressed version.
        let mut buf = Vec::with_capacity(asdf.len());
        brotli::BrotliDecompress(&mut asdf, &mut buf).map(|()| buf)
      })
      .and_then(Result::ok)
      .map(Cow::Owned)
  }

  /// Get an asset by key.
  #[cfg(not(feature = "compression"))]
  pub fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    self
      .assets
      .get(key.as_ref())
      .copied()
      .map(|a| Cow::Owned(a.to_vec()))
  }

  /// Iterate on the assets.
  pub fn iter(&self) -> Box<AssetsIter<'_>> {
    Box::new(
      self
        .assets
        .into_iter()
        .map(|(k, b)| (Cow::Borrowed(*k), Cow::Borrowed(*b))),
    )
  }

  /// CSP hashes for the given asset.
  pub fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(
      self
        .global_hashes
        .iter()
        .chain(
          self
            .html_hashes
            .get(html_path.as_ref())
            .copied()
            .into_iter()
            .flatten(),
        )
        .copied(),
    )
  }
}
