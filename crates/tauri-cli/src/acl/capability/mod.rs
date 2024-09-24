// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::{Parser, Subcommand};

use crate::Result;

mod new;

#[derive(Debug, Parser)]
#[clap(about = "Manage or create capabilities for your app")]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  #[clap(alias = "create")]
  New(new::Options),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::New(options) => new::command(options),
  }
}
