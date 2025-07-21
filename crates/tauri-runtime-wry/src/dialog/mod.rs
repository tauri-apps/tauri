// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
mod windows;

pub fn error<S: AsRef<str>>(err: S) {
  #[cfg(windows)]
  windows::error(err);

  #[cfg(not(windows))]
  {
    unimplemented!("Error dialog is not implemented for this platform");
  }
}
