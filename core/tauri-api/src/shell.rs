// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/// Open path or URL with `with`, or system default
pub fn open(path: String, with: Option<String>) -> crate::Result<()> {
  {
    let exit_status = if let Some(with) = with {
      open::with(&path, &with)
    } else {
      open::that(&path)
    };
    match exit_status {
      Ok(status) => {
        if status.success() {
          Ok(())
        } else {
          Err(crate::Error::Shell("open command failed".into()))
        }
      }
      Err(err) => Err(crate::Error::Shell(format!(
        "failed to open: {}",
        err.to_string()
      ))),
    }
  }
}
