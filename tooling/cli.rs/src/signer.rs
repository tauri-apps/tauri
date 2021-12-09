// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::{AppSettings, Parser, Subcommand};

mod generate;
mod sign;

#[derive(Parser)]
#[clap(author, version, about = "Tauri updater signer")]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Sign(sign::Options),
  Generate(generate::Options),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Sign(options) => sign::command(options)?,
    Commands::Generate(options) => generate::command(options)?,
  }
  Ok(())
}
