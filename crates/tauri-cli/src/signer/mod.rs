// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::{Parser, Subcommand};

mod android;
mod apple;
mod generate;
mod secrets;
mod sign;
mod windows;

#[derive(Parser)]
#[clap(
  author,
  version,
  about = "Generate updater signing keys, sign files and set up code signing certificates for distribution",
  subcommand_required(true),
  arg_required_else_help(true)
)]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Sign(sign::Options),
  Generate(generate::Options),
  Android(android::Cli),
  Apple(apple::Cli),
  Windows(windows::Cli),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Sign(options) => sign::command(options)?,
    Commands::Generate(options) => generate::command(options)?,
    Commands::Android(cli) => android::command(cli)?,
    Commands::Apple(cli) => apple::command(cli)?,
    Commands::Windows(cli) => windows::command(cli)?,
  }
  Ok(())
}
