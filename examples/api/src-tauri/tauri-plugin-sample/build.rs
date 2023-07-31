// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;
use tauri_build::{
  mobile::PluginBuilder,
  plugin::{set_manifest, Manifest, ScopeType},
};

fn main() {
  if let Err(error) = PluginBuilder::new()
    .android_path("android")
    .ios_path("ios")
    .run()
  {
    println!("{error:#}");
    exit(1);
  }

  set_manifest(
    Manifest::new("sample")
      .default_capability_json(include_str!("capabilities/default.json"))
      .capability_json(include_str!("capabilities/ping.json"))
      .feature("ping")
      .scope_type(ScopeType::String),
  );
}
