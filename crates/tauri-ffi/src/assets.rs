// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Runtime `Assets` implementations — the FFI replacement for the
//! compile-time `EmbeddedAssets` produced by `generate_context!`.

use std::borrow::Cow;
use std::path::{Component, Path, PathBuf};

use tauri::utils::assets::{AssetKey, AssetsIter, CspHash};
use tauri::{Assets, Runtime};

/// Serves the app frontend from a directory on disk.
///
/// TODO(M1): percent-decode keys, stream large files, optional watch mode.
pub struct DirAssets {
  root: PathBuf,
}

impl DirAssets {
  pub fn new(root: PathBuf) -> Self {
    Self { root }
  }

  fn resolve(&self, key: &str) -> Option<PathBuf> {
    let relative = key.trim_start_matches('/');
    let relative = if relative.is_empty() {
      "index.html"
    } else {
      relative
    };
    let relative = Path::new(relative);
    // Reject anything that could escape the root (`..`, absolute paths).
    if relative
      .components()
      .any(|component| !matches!(component, Component::Normal(_)))
    {
      return None;
    }
    let path = self.root.join(relative);
    if path.is_dir() {
      let index = path.join("index.html");
      return index.is_file().then_some(index);
    }
    path.is_file().then_some(path)
  }
}

impl<R: Runtime> Assets<R> for DirAssets {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    self
      .resolve(key.as_ref())
      .and_then(|path| std::fs::read(path).ok())
      .map(Cow::Owned)
  }

  fn iter(&self) -> Box<AssetsIter<'_>> {
    let mut files = Vec::new();
    collect(&self.root, &self.root, &mut files);
    Box::new(
      files
        .into_iter()
        .map(|(key, bytes)| (Cow::Owned(key), Cow::Owned(bytes))),
    )
  }

  fn csp_hashes(&self, _html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(std::iter::empty())
  }
}

fn collect(root: &Path, dir: &Path, out: &mut Vec<(String, Vec<u8>)>) {
  let Ok(entries) = std::fs::read_dir(dir) else {
    return;
  };
  for entry in entries.flatten() {
    let path = entry.path();
    if path.is_dir() {
      collect(root, &path, out);
    } else if let (Ok(relative), Ok(bytes)) = (path.strip_prefix(root), std::fs::read(&path)) {
      out.push((
        format!("/{}", relative.to_string_lossy().replace('\\', "/")),
        bytes,
      ));
    }
  }
}

/// No frontend assets (every window points at a remote URL).
pub struct EmptyAssets;

impl<R: Runtime> Assets<R> for EmptyAssets {
  fn get(&self, _key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    None
  }

  fn iter(&self) -> Box<AssetsIter<'_>> {
    Box::new(std::iter::empty())
  }

  fn csp_hashes(&self, _html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(std::iter::empty())
  }
}
