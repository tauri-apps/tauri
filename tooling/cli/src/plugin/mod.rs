// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

use crate::Result;

mod android;
mod init;
mod ios;

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
  Init(init::Options),
  Android(android::Cli),
  Ios(ios::Cli),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Init(options) => init::command(options)?,
    Commands::Android(cli) => android::command(cli)?,
    Commands::Ios(cli) => ios::command(cli)?,
  }

  Ok(())
}
