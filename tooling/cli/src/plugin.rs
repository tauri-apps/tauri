// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{AppSettings, Parser, Subcommand};

use crate::Result;

mod init;

#[derive(Parser)]
#[clap(author, version, about = "Manage Tauri plugins")]
#[clap(setting(AppSettings::SubcommandRequiredElseHelp))]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Init(init::Options),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init::command(options)?,
  }

  Ok(())
}
