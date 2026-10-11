// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use ctor::declarative::ctor;
use std::{
  io::{Error, ErrorKind, Result},
  path::{Path, PathBuf},
};

ctor! {
  /// A cached version of the current binary using [`ctor`] to cache it before even `main` runs.
  #[ctor(unsafe)]
  pub(super) static STARTING_BINARY: StartingBinary = unsafe { StartingBinary::new() };
}

/// Represents a binary path that was cached when the program was loaded.
pub(super) struct StartingBinary(std::io::Result<PathBuf>);

impl StartingBinary {
  /// Find the starting executable as safely as possible.
  fn new() -> Self {
    // see notes on current_exe() for security implications
    let dangerous_path = match std::env::current_exe() {
      Ok(dangerous_path) => dangerous_path,
      error @ Err(_) => return Self(error),
    };

    // note: this only checks symlinks on problematic platforms, see implementation below
    if let Some(symlink) = Self::has_symlink(&dangerous_path) {
      return Self(Err(Error::new(
        ErrorKind::InvalidData,
        format!(
          "StartingBinary found current_exe() that contains a symlink on a non-allowed platform: {}",
          symlink.display()
        ),
      )));
    }

    // we canonicalize the path to resolve any symlinks to the real exe path
    let path = dangerous_path.canonicalize();
    #[cfg(target_os = "macos")]
    let path = path.map(|path| Self::dev_bundle_origin(&path).unwrap_or(path));
    Self(path)
  }

  /// In development, macOS apps run from a bundle that hard links the binary in the cargo output,
  /// `<target dir>/.tauri-dev/<binary>.app/Contents/MacOS/<binary>`; resolve to that binary instead.
  #[cfg(target_os = "macos")]
  fn dev_bundle_origin(path: &Path) -> Option<PathBuf> {
    use std::os::unix::fs::MetadataExt;

    let bin_name = path.file_name()?;
    let macos_dir = path.parent()?;
    let contents_dir = macos_dir.parent()?;
    let bundle_dir = contents_dir.parent()?;
    let bundle_parent = bundle_dir.parent()?;
    if macos_dir.file_name()? != "MacOS"
      || contents_dir.file_name()? != "Contents"
      || bundle_dir.extension()? != "app"
      || bundle_parent.file_name()? != ".tauri-dev"
    {
      return None;
    }

    let origin = bundle_parent.parent()?.join(bin_name);
    let (a, b) = (path.metadata().ok()?, origin.metadata().ok()?);
    (a.dev() == b.dev() && a.ino() == b.ino()).then_some(origin)
  }

  /// A clone of the [`PathBuf`] found to be the starting path.
  ///
  /// Because [`Error`] is not clone-able, it is recreated instead.
  pub(super) fn cloned(&self) -> Result<PathBuf> {
    // false positive
    #[allow(clippy::useless_asref)]
    self
      .0
      .as_ref()
      .map(Clone::clone)
      .map_err(|e| Error::new(e.kind(), e.to_string()))
  }

  /// We only care about checking this on macOS currently, as it has the least symlink protections.
  #[cfg(any(
    not(target_os = "macos"),
    feature = "process-relaunch-dangerous-allow-symlink-macos"
  ))]
  fn has_symlink(_: &Path) -> Option<&Path> {
    None
  }

  /// We only care about checking this on macOS currently, as it has the least symlink protections.
  #[cfg(all(
    target_os = "macos",
    not(feature = "process-relaunch-dangerous-allow-symlink-macos")
  ))]
  fn has_symlink(path: &Path) -> Option<&Path> {
    path.ancestors().find(|ancestor| {
      matches!(
        ancestor
          .symlink_metadata()
          .as_ref()
          .map(std::fs::Metadata::file_type)
          .as_ref()
          .map(std::fs::FileType::is_symlink),
        Ok(true)
      )
    })
  }
}
