// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Parser;

use crate::{
  acl,
  helpers::{
    app_paths::{resolve_frontend_dir, tauri_dir},
    cargo,
    npm::PackageManager,
  },
  Result,
};

#[derive(Debug, Parser)]
#[clap(about = "Remove a tauri plugin from the project")]
pub struct Options {
  /// The plugin to remove.
  pub plugin: String,
}

pub fn command(options: Options) -> Result<()> {
  crate::helpers::app_paths::resolve();
  run(options)
}

pub fn run(options: Options) -> Result<()> {
  let plugin = options.plugin;

  let crate_name = format!("tauri-plugin-{plugin}");

  let mut plugins = crate::helpers::plugins::known_plugins();
  let metadata = plugins.remove(plugin.as_str()).unwrap_or_default();

  let frontend_dir = resolve_frontend_dir();
  let tauri_dir = tauri_dir();

  let target_str = metadata
    .desktop_only
    .then_some(r#"cfg(not(any(target_os = "android", target_os = "ios")))"#)
    .or_else(|| {
      metadata
        .mobile_only
        .then_some(r#"cfg(any(target_os = "android", target_os = "ios"))"#)
    });

  cargo::uninstall_one(cargo::CargoUninstallOptions {
    name: &crate_name,
    cwd: Some(tauri_dir),
    target: target_str,
  })?;

  if !metadata.rust_only {
    if let Some(manager) = frontend_dir.map(PackageManager::from_project) {
      let npm_name = format!("@tauri-apps/plugin-{plugin}");
      manager.remove(&[npm_name], tauri_dir)?;
    }

    acl::permission::rm::command(acl::permission::rm::Options {
      identifier: format!("{plugin}:*"),
    })?;
  }

  log::info!("Now, you must manually remove the plugin from your Rust code.",);

  Ok(())
}
