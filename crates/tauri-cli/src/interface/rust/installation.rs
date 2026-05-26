// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(target_os = "macos", target_os = "linux", windows))]
use crate::{
  error::{Error, ErrorExt},
  Result,
};

#[cfg(any(target_os = "macos", target_os = "linux", windows))]
use std::{fs::read_dir, path::PathBuf, process::Command};

#[cfg(any(target_os = "macos", target_os = "linux", windows))]
pub fn installed_targets() -> Result<Vec<String>> {
  let output = Command::new("rustc")
    .args(["--print", "sysroot"])
    .output()
    .map_err(|error| Error::CommandFailed {
      command: "rustc --print sysroot".to_string(),
      error,
    })?;
  let sysroot_path = PathBuf::from(String::from_utf8_lossy(&output.stdout).trim().to_string());

  let mut targets = Vec::new();
  for entry in read_dir(sysroot_path.join("lib").join("rustlib"))
    .fs_context(
      "failed to read Rust sysroot",
      sysroot_path.join("lib").join("rustlib"),
    )?
    .flatten()
  {
    if entry.file_type().map(|t| t.is_dir()).unwrap_or_default() {
      let name = entry.file_name();
      if name != "etc" && name != "src" {
        targets.push(name.to_string_lossy().into_owned());
      }
    }
  }

  Ok(targets)
}
