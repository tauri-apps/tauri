// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

use crate::Result;

pub mod add;
mod ls;
mod new;
pub mod rm;

#[derive(Debug, Parser)]
#[clap(about = "Manage or create permissions for your app or plugin")]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[clap(alias = "create")]
  New(new::Options),
  Add(add::Options),
  #[clap(alias = "remove")]
  Rm(rm::Options),
  #[clap(alias = "list")]
  Ls(ls::Options),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::New(options) => new::command(options),
    Commands::Add(options) => add::command(options),
    Commands::Rm(options) => rm::command(options),
    Commands::Ls(options) => ls::command(options),
  }
}
