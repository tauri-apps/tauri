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

/// Serves the app frontend from a tauri-ffi assets archive — a single file
/// packed by `tauri build` for bindings projects, the embedded-assets
/// equivalent for apps that are not compiled.
///
/// Format (little-endian): 8-byte magic `TAURIPK1`, `u64` index length,
/// index JSON (`{"files":{"/path":[offset,len]}}` with offsets relative to
/// the blob region that follows the index), concatenated file contents.
pub struct ArchiveAssets {
  /// The blob region (everything after the index).
  blob: Vec<u8>,
  /// Asset key (`/index.html`) -> (offset, len) into `blob`.
  index: std::collections::HashMap<String, (usize, usize)>,
}

impl ArchiveAssets {
  const MAGIC: &'static [u8; 8] = b"TAURIPK1";

  pub fn load(path: &Path) -> Result<Self, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("failed to read assets archive: {e}"))?;
    Self::from_bytes(bytes)
  }

  /// Parses an archive already held in memory — the embedded-in-binary path,
  /// where the host reads the archive from its own executable (Deno
  /// `--include`, Node SEA asset, PyInstaller data) and hands over the bytes.
  pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, String> {
    if bytes.len() < 16 || &bytes[..8] != Self::MAGIC {
      return Err("not a tauri-ffi assets archive (bad magic)".into());
    }
    let index_len = u64::from_le_bytes(bytes[8..16].try_into().unwrap()) as usize;
    let blob_start = 16 + index_len;
    if bytes.len() < blob_start {
      return Err("corrupt assets archive: truncated index".into());
    }

    #[derive(serde::Deserialize)]
    struct Index {
      files: std::collections::HashMap<String, (u64, u64)>,
    }
    let index: Index = serde_json::from_slice(&bytes[16..blob_start])
      .map_err(|e| format!("corrupt assets archive index: {e}"))?;

    let blob = bytes[blob_start..].to_vec();
    let mut files = std::collections::HashMap::with_capacity(index.files.len());
    for (key, (offset, len)) in index.files {
      let (offset, len) = (offset as usize, len as usize);
      if offset + len > blob.len() {
        return Err(format!("corrupt assets archive: `{key}` is out of bounds"));
      }
      files.insert(key, (offset, len));
    }

    Ok(Self { blob, index: files })
  }

  fn get_bytes(&self, key: &str) -> Option<&[u8]> {
    let key = if key == "/" || key.is_empty() {
      "/index.html"
    } else {
      key
    };
    let slice = |&(offset, len): &(usize, usize)| &self.blob[offset..offset + len];
    if let Some(range) = self.index.get(key) {
      return Some(slice(range));
    }
    // directory-style fallback, like DirAssets
    self.index.get(&format!("{key}/index.html")).map(slice)
  }
}

impl<R: Runtime> Assets<R> for ArchiveAssets {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    self.get_bytes(key.as_ref()).map(Cow::Borrowed)
  }

  fn iter(&self) -> Box<AssetsIter<'_>> {
    Box::new(self.index.iter().map(|(key, &(offset, len))| {
      (
        Cow::Borrowed(key.as_str()),
        Cow::Borrowed(&self.blob[offset..offset + len]),
      )
    }))
  }

  fn csp_hashes(&self, _html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(std::iter::empty())
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
