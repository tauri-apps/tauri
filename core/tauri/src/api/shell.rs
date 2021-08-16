// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to shell.

/// Opens path or URL with `with`, or system default
pub fn open(path: String, with: Option<String>) -> crate::api::Result<()> {
  {
    let exit_status = if let Some(with) = with {
      open::with(&path, &with)
    } else {
      open::that(&path)
    };
    exit_status
      .map_err(|err| crate::api::Error::Shell(format!("failed to open: {}", err.to_string())))
  }
}
