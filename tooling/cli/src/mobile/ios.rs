// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::config::get as get_tauri_config;
use cargo_mobile::{
  apple::config::{Config as AppleConfig, Metadata as AppleMetadata},
  config::{metadata::Metadata, Config},
  os,
};
use clap::{Parser, Subcommand};

use super::{
  ensure_init,
  init::{command as init_command, Options as InitOptions},
  Target,
};
use crate::Result;

pub(crate) mod project;

#[derive(Debug, thiserror::Error)]
enum Error {
  #[error("invalid tauri configuration: {0}")]
  InvalidTauriConfig(String),
  #[error("{0}")]
  ProjectNotInitialized(String),
  #[error(transparent)]
  OpenFailed(os::OpenFileError),
}

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "iOS commands",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Init(InitOptions),
  Open,
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, Target::Ios)?,
    Commands::Open => open()?,
  }

  Ok(())
}

fn with_config(
  f: impl FnOnce(&AppleConfig, &AppleMetadata) -> Result<(), Error>,
) -> Result<(), Error> {
  let config = get_tauri_config(None).map_err(|e| Error::InvalidTauriConfig(e.to_string()))?;
  let config_guard = config.lock().unwrap();
  let config_ = config_guard.as_ref().unwrap();
  let mobile_config = get_config(config_);
  let metadata = get_metadata(config_);
  f(config.apple(), metadata.apple())
}

fn open() -> Result<()> {
  with_config(|config, _metadata| {
    ensure_init(config.project_dir(), Target::Ios)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    os::open_file_with("Xcode", config.project_dir()).map_err(Error::OpenFailed)
  })?;
  Ok(())
}
