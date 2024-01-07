// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;

fn main() {
  if let Err(error) = tauri_build::mobile::PluginBuilder::new()
    .android_path("android")
    .ios_path("ios")
    .run()
  {
    println!("{error:#}");
    exit(1);
  }

  tauri_plugin::build("./permissions/**/*.toml");
}
