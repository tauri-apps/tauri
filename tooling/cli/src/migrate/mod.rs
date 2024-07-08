// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::{app_dir, tauri_dir},
    npm::PackageManager,
  },
  Result,
};
use anyhow::Context;

mod config;
mod frontend;
mod manifest;
mod v1_plugins;

pub fn command() -> Result<()> {
  let tauri_dir = tauri_dir();
  let app_dir = app_dir();

  let migrated = config::migrate(&tauri_dir).context("Could not migrate config")?;
  manifest::migrate(&tauri_dir).context("Could not migrate manifest")?;
  let v1_plugins = v1_plugins::migrate(&tauri_dir).context("Could not migrate v1 plugins")?;
  let frontend_plugins = frontend::migrate(app_dir).context("Could not migrate frontend")?;

  // Add plugins
  let mut plugins = migrated.plugins;
  plugins.extend(frontend_plugins);
  plugins.extend(v1_plugins.plugins);
  for plugin in plugins {
    let features = v1_plugins.cargo_features.get(plugin.as_str());

    crate::add::command(crate::add::Options {
      plugin: plugin.clone(),
      branch: None,
      tag: None,
      rev: None,
      features: features.map(ToString::to_string),
      ..Default::default()
    })
    .with_context(|| format!("Could not add '{plugin}' plugin"))?
  }

  // Update @tauri-apps/api version
  let pm = PackageManager::from_project(app_dir)
    .into_iter()
    .next()
    .unwrap_or(PackageManager::Npm);
  pm.install(&["@tauri-apps/api@>=2.0.0-beta.0".into()])
    .context("Failed to update @tauri-apps/api package to v2")?;

  Ok(())
}
