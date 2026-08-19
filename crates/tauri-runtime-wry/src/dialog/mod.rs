// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
mod windows;

// Takes a `&'static str` here since we convert clickable hyperlinks,
// DO NOT pass in untrusted input
#[cfg_attr(not(windows), allow(unused))]
pub fn error(err: &'static str) {
  #[cfg(windows)]
  windows::error(err);

  #[cfg(not(windows))]
  {
    unimplemented!("Error dialog is not implemented for this platform");
  }
}
