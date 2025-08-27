// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::path::{Path, PathBuf};

pub fn copy_file(from: impl AsRef<Path>, to: impl AsRef<Path>) -> Result<()> {
  let from = from.as_ref();
  let to = to.as_ref();
  if !from.exists() {
    return Err(anyhow::anyhow!("{:?} does not exist", from));
  }
  if !from.is_file() {
    return Err(anyhow::anyhow!("{:?} is not a file", from));
  }
  let dest_dir = to.parent().expect("No data in parent");
  std::fs::create_dir_all(dest_dir)?;
  std::fs::copy(from, to)?;
  Ok(())
}

pub fn find_in_directory(path: &Path, glob_pattern: &str) -> Result<PathBuf> {
  let pattern = glob::Pattern::new(glob_pattern)?;
  for entry in std::fs::read_dir(path)? {
    let entry = entry?;
    if pattern.matches_path(&entry.path()) {
      return Ok(entry.path());
    }
  }
  Err(anyhow::anyhow!(
    "No file found in {} matching {}",
    path.display(),
    glob_pattern
  ))
}
