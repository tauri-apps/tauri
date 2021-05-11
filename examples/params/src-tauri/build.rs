// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_build::{try_build, Attributes, WindowsAttributes};

fn main() {
  if let Err(error) = try_build(
    Attributes::new()
      .windows_attributes(WindowsAttributes::new().window_icon_path("../../.icons/icon.ico")),
  ) {
    panic!("error found during tauri-build: {}", error);
  }
}
