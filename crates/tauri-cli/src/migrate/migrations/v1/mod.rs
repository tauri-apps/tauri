// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{error::Context, helpers::app_paths::Dirs, Result};

mod config;
mod frontend;
mod manifest;

pub fn run(dirs: &Dirs) -> Result<()> {
  let mut migrated = config::migrate(dirs.tauri).context("Could not migrate config")?;
  manifest::migrate(dirs.tauri).context("Could not migrate manifest")?;
  let plugins = frontend::migrate(dirs.frontend)?;

  migrated.plugins.extend(plugins);

  // Add plugins
  for plugin in migrated.plugins {
    crate::add::run(
      crate::add::Options {
        plugin: plugin.clone(),
        branch: None,
        tag: None,
        rev: None,
        no_fmt: false,
      },
      dirs,
    )
    .with_context(|| format!("Could not migrate plugin '{plugin}'"))?;
  }

  Ok(())
}
