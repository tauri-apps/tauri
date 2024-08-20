// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Result;
use std::path::Path;

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
