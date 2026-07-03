// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::{Context, ErrorExt},
  Error,
};
use std::path::{Path, PathBuf};

pub async fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> crate::Result<()> {
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
  tokio::fs::create_dir_all(dest_dir)
    .await
    .fs_context("failed to create directory", dest_dir.to_path_buf())?;
  tokio::fs::copy(from, to)
    .await
    .fs_context("failed to copy file", from.to_path_buf())?;
  Ok(())
}

/// Find an entry in a directory matching a glob pattern.
/// Currently does not traverse subdirectories.
// currently only used on macOS
#[allow(dead_code)]
pub async fn find_in_directory(path: &Path, glob_pattern: &str) -> crate::Result<PathBuf> {
  let pattern = glob::Pattern::new(glob_pattern)
    .with_context(|| format!("failed to parse glob pattern {glob_pattern}"))?;
  let mut entries = tokio::fs::read_dir(path)
    .await
    .with_context(|| format!("failed to read directory {}", path.display()))?;
  while let Some(entry) = entries
    .next_entry()
    .await
    .context("failed to read directory entry")?
  {
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
