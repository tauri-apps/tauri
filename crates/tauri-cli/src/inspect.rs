// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT
use std::path::Path;

use crate::Result;
use clap::{Parser, Subcommand};

use crate::interface::{AppInterface, AppSettings};

#[derive(Debug, Parser)]
#[clap(about = "Inspect values used by Tauri")]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  /// Print the default Upgrade Code used by MSI installer derived from productName.
  WixUpgradeCode,
}

pub fn command(cli: Cli) -> Result<()> {
  let dirs = crate::helpers::app_paths::resolve_dirs();
  match cli.command {
    Commands::WixUpgradeCode => wix_upgrade_code(dirs.tauri),
  }
}

// NOTE: if this is ever changed, make sure to also update Wix upgrade code generation in tauri-bundler
fn wix_upgrade_code(tauri_dir: &Path) -> Result<()> {
  let target = tauri_utils::platform::Target::Windows;
  let config = crate::helpers::config::get_config(target, &[], tauri_dir)?;

  let interface = AppInterface::new(&config, None, tauri_dir)?;

  let product_name = interface.app_settings().get_package_settings().product_name;

  let upgrade_code = uuid::Uuid::new_v5(
    &uuid::Uuid::NAMESPACE_DNS,
    format!("{product_name}.exe.app.x64").as_bytes(),
  );

  log::info!("Default WiX Upgrade Code, derived from {product_name}: {upgrade_code}");
  if let Some(code) = config
    .bundle
    .windows
    .wix
    .as_ref()
    .and_then(|wix| wix.upgrade_code)
  {
    log::info!("Application Upgrade Code override: {code}");
  }

  Ok(())
}
