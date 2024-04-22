// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use std::{fs, path::Path};

use anyhow::Context;

use crate::{
  helpers::{cargo, npm::PackageManager},
  Result,
};

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

pub fn migrate(tauri_dir: &Path, app_dir: &Path) -> Result<()> {
  let manifest_path = tauri_dir.join("Cargo.toml");
  let manifest = fs::read_to_string(manifest_path).context("failed to read Cargo.toml")?;

  let plugins_to_migrate = PLUGINS
    .iter()
    .filter(|p| manifest.contains(&format!("tauri-plugin-{p}")));

  let cargo_deps = plugins_to_migrate
    .clone()
    .map(|p| format!("tauri-plugin-{p}@2.0.0-beta"))
    .collect::<Vec<_>>();
  cargo::install(&cargo_deps, Some(tauri_dir))?;

  let npm_deps = plugins_to_migrate
    .map(|p| format!("@tauri-apps/plugin-{p}@>=2.0.0-beta.0"))
    .collect::<Vec<_>>();
  let pm = PackageManager::from_project(app_dir)
    .into_iter()
    .next()
    .unwrap_or(PackageManager::Npm);
  pm.install(&npm_deps)?;

  Ok(())
}
