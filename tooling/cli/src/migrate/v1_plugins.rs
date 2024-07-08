// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use std::{collections::HashMap, fs, path::Path};

use anyhow::Context;

use crate::Result;

const PLUGINS: &[&str] = &[
  "authenticator",
  "autostart",
  "fs-extra",
  "fs-watch",
  "localhost",
  "log",
  "persisted-scope",
  "positioner",
  "single-instance",
  "sql",
  "store",
  "stronghold",
  "upload",
  "websocket",
  "window-state",
];

pub struct V1Plugins {
  pub plugins: Vec<String>,
  pub cargo_features: HashMap<&'static str, &'static str>,
}

pub fn migrate(tauri_dir: &Path) -> Result<V1Plugins> {
  let manifest_path = tauri_dir.join("Cargo.toml");
  let manifest = fs::read_to_string(manifest_path).context("failed to read Cargo.toml")?;

  let mut cargo_features = HashMap::new();

  let plugins = PLUGINS
    .iter()
    .filter(|p| manifest.contains(&format!("tauri-plugin-{p}")))
    .clone()
    .map(|p| match *p {
      "fs-extra" => "fs".to_string(),
      "fs-watch" => {
        cargo_features.insert("fs", "tauri-plugin-fs/watch");
        "fs".to_string()
      }
      _ => p.to_string(),
    })
    .collect::<Vec<_>>();

  Ok(V1Plugins {
    plugins,
    cargo_features,
  })
}
