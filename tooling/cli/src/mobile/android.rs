// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use cargo_mobile::{
  android::config::{Config as AndroidConfig, Metadata as AndroidMetadata},
  config::{metadata::Metadata, Config},
  os,
  util::cli::TextWrapper,
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
  #[error("{0}")]
  ProjectNotInitialized(String),
  #[error(transparent)]
  ConfigFailed(cargo_mobile::config::LoadOrGenError),
  #[error(transparent)]
  MetadataFailed(cargo_mobile::config::metadata::Error),
  #[error("Android is marked as unsupported in your configuration file")]
  Unsupported,
  #[error(transparent)]
  OpenFailed(os::OpenFileError),
}

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Android commands",
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
    Commands::Init(options) => init_command(options, Target::Android)?,
    Commands::Open => open()?,
  }

  Ok(())
}

fn with_config(
  wrapper: &TextWrapper,
  f: impl FnOnce(&AndroidConfig, &AndroidMetadata) -> Result<(), Error>,
) -> Result<(), Error> {
  let (config, _origin) =
    Config::load_or_gen(".", true.into(), wrapper).map_err(Error::ConfigFailed)?;
  let metadata = Metadata::load(&config.app().root_dir()).map_err(Error::MetadataFailed)?;
  if metadata.android().supported() {
    f(config.android(), metadata.android())
  } else {
    Err(Error::Unsupported)
  }
}

fn open() -> Result<()> {
  let wrapper = TextWrapper::with_splitter(textwrap::termwidth(), textwrap::NoHyphenation);
  with_config(&wrapper, |config, _metadata| {
    ensure_init(config.project_dir(), Target::Android)
      .map_err(|e| Error::ProjectNotInitialized(e.to_string()))?;
    os::open_file_with("Android Studio", config.project_dir()).map_err(Error::OpenFailed)
  })?;
  Ok(())
}
