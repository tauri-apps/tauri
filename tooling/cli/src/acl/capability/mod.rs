use clap::{Parser, Subcommand};

use crate::Result;

mod create;

#[derive(Debug, Parser)]
#[clap(about = "Manage or create capabilities for your app")]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  Create(create::Options),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Create(options) => create::command(options),
  }
}
