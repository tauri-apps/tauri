// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fmt::Display, path::Path};

use clap::{Parser, Subcommand, ValueEnum};

use crate::Result;

mod android;
mod init;
mod ios;
mod new;

#[derive(Debug, Clone, ValueEnum, Default)]
pub enum PluginIosFramework {
  /// Swift Package Manager project
  #[default]
  Spm,
  /// Xcode project
  Xcode,
}

impl Display for PluginIosFramework {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      Self::Spm => write!(f, "spm"),
      Self::Xcode => write!(f, "xcode"),
    }
  }
}

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Manage or create Tauri plugins",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  New(new::Options),
  Init(init::Options),
  Android(android::Cli),
  Ios(ios::Cli),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::New(options) => new::command(options)?,
    Commands::Init(options) => init::command(options)?,
    Commands::Android(cli) => android::command(cli)?,
    Commands::Ios(cli) => ios::command(cli)?,
  }

  Ok(())
}

fn infer_plugin_name<P: AsRef<Path>>(directory: P) -> Result<String> {
  let dir = directory.as_ref();
  let cargo_toml_path = dir.join("Cargo.toml");
  let name = if cargo_toml_path.exists() {
    let contents = std::fs::read_to_string(cargo_toml_path)?;
    let cargo_toml: toml::Value = toml::from_str(&contents)?;
    cargo_toml
      .get("package")
      .and_then(|v| v.get("name"))
      .map(|v| v.as_str().unwrap_or_default())
      .unwrap_or_default()
      .to_string()
  } else {
    dir
      .file_name()
      .unwrap_or_default()
      .to_string_lossy()
      .to_string()
  };
  Ok(
    name
      .strip_prefix("tauri-plugin-")
      .unwrap_or(&name)
      .to_string(),
  )
}
