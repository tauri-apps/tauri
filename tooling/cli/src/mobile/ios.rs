// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

use super::init::{command as init_command, Options as InitOptions, Target as InitTarget};
use crate::Result;

pub(crate) mod project;

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
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init_command(options, InitTarget::Ios)?,
  }

  Ok(())
}
