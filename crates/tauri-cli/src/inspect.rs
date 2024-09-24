// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::interface::{AppInterface, AppSettings, Interface};

#[derive(Debug, Parser)]
#[clap(about = "Manage or create permissions for your app or plugin")]
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
  match cli.command {
    Commands::WixUpgradeCode => wix_upgrade_code(),
  }
}

// NOTE: if this is ever changed, make sure to also update Wix upgrade code generation in tauri-bundler
fn wix_upgrade_code() -> Result<()> {
  crate::helpers::app_paths::resolve();

  let target = tauri_utils::platform::Target::Windows;
  let config = crate::helpers::config::get(target, None)?;

  let interface = AppInterface::new(config.lock().unwrap().as_ref().unwrap(), None)?;

  let product_name = interface.app_settings().get_package_settings().product_name;

  let upgrade_code = uuid::Uuid::new_v5(
    &uuid::Uuid::NAMESPACE_DNS,
    format!("{product_name}.exe.app.x64").as_bytes(),
  )
  .to_string();

  log::info!("Default WiX Upgrade Code, derived from {product_name}: {upgrade_code}");
  if let Some(code) = config.lock().unwrap().as_ref().and_then(|c| {
    c.bundle
      .windows
      .wix
      .as_ref()
      .and_then(|wix| wix.upgrade_code)
  }) {
    log::info!("Application Upgrade Code override: {code}");
  }

  Ok(())
}
