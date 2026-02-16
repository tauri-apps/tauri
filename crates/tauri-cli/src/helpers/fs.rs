// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  Error,
  error::{Context, ErrorExt},
};
use std::path::{Path, PathBuf};

pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> crate::Result<()> {
  let from = from.as_ref();
  let to = to.as_ref();
  if !from.exists() {
    Err(Error::Fs {
      context: "failed to copy file",
      path: from.to_path_buf(),
      error: std::io::Error::new(std::io::ErrorKind::NotFound, "source does not exist"),
    })?;
  }
  if !from.is_file() {
    Err(Error::Fs {
      context: "failed to copy file",
      path: from.to_path_buf(),
      error: std::io::Error::other("not a file"),
    })?;
  }
  let dest_dir = to.parent().expect("No data in parent");
  std::fs::create_dir_all(dest_dir)
    .fs_context("failed to create directory", dest_dir.to_path_buf())?;
  std::fs::copy(from, to).fs_context("failed to copy file", from.to_path_buf())?;
  Ok(())
}

/// Find an entry in a directory matching a glob pattern.
/// Currently does not traverse subdirectories.
// currently only used on macOS
#[allow(dead_code)]
pub fn find_in_directory(path: &Path, glob_pattern: &str) -> crate::Result<PathBuf> {
  let pattern = glob::Pattern::new(glob_pattern)
    .with_context(|| format!("failed to parse glob pattern {glob_pattern}"))?;
  for entry in std::fs::read_dir(path)
    .with_context(|| format!("failed to read directory {}", path.display()))?
  {
    let entry = entry.context("failed to read directory entry")?;
    if pattern.matches_path(&entry.path()) {
      return Ok(entry.path());
    }
  }
  crate::error::bail!(
    "No file found in {} matching {}",
    path.display(),
    glob_pattern
  )
}

#[allow(dead_code)]
pub fn copy_dir_all(src: &Path, dst: &Path) -> crate::Result<()> {
  std::fs::create_dir_all(dst).fs_context("failed to create directory", dst.to_path_buf())?;
  for entry in std::fs::read_dir(src).fs_context("failed to read directory", src.to_path_buf())? {
    let entry = entry.map_err(|e| crate::Error::GenericError(e.to_string()))?;
    let dst_path = dst.join(entry.file_name());
    let file_type = entry
      .file_type()
      .map_err(|e| crate::Error::GenericError(e.to_string()))?;
    if file_type.is_dir() {
      copy_dir_all(&entry.path(), &dst_path)?;
    } else {
      std::fs::copy(entry.path(), &dst_path)
        .map_err(|e| crate::Error::GenericError(e.to_string()))?;
    }
  }
  Ok(())
}
