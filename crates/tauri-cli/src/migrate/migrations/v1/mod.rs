// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::app_paths::{frontend_dir, tauri_dir},
  Result,
};

use anyhow::Context;

mod config;
mod frontend;
mod manifest;

pub fn run() -> Result<()> {
  let tauri_dir = tauri_dir();
  let frontend_dir = frontend_dir();

  let mut migrated = config::migrate(tauri_dir).context("Could not migrate config")?;
  manifest::migrate(tauri_dir).context("Could not migrate manifest")?;
  let plugins = frontend::migrate(frontend_dir)?;

  migrated.plugins.extend(plugins);

  // Add plugins
  for plugin in migrated.plugins {
    crate::add::run(crate::add::Options {
      plugin: plugin.clone(),
      branch: None,
      tag: None,
      rev: None,
      no_fmt: false,
    })
    .with_context(|| format!("Could not migrate plugin '{plugin}'"))?;
  }

  Ok(())
}
