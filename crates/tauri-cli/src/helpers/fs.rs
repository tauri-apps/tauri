// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{error::ErrorExt, Error};
use std::path::Path;

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
