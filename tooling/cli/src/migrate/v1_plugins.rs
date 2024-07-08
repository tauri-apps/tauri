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

  let mut features = Vec::new();

  let plugins_to_migrate = PLUGINS
    .iter()
    .filter(|p| manifest.contains(&format!("tauri-plugin-{p}")));

  let mut cargo_deps = plugins_to_migrate
    .clone()
    .map(|p| match *p {
      "fs-extra" => "fs",
      "fs-watch" => {
        features.push(format!("tauri-plugin-{p}/watch"));
        "fs"
      }
      _ => p,
    })
    .map(|p| format!("tauri-plugin-{p}@2.0.0-beta"))
    .collect::<Vec<_>>();

  cargo_deps.sort();
  cargo_deps.dedup();

  cargo::add(
    &cargo_deps,
    cargo::AddOptions {
      cwd: Some(tauri_dir),
      features: Some(features),
    },
  )?;

  if app_dir.join("package.json").exists() {
    let mut npm_deps = plugins_to_migrate
      .map(|p| match *p {
        "fs-extra" | "fs-watch" => "fs",
        _ => p,
      })
      .map(|p| format!("@tauri-apps/plugin-{p}@>=2.0.0-beta.0"))
      .collect::<Vec<_>>();

    npm_deps.sort();
    npm_deps.dedup();

    let pm = PackageManager::from_project(app_dir)
      .into_iter()
      .next()
      .unwrap_or(PackageManager::Npm);

    pm.install(&npm_deps)?;
  }

  Ok(())
}
