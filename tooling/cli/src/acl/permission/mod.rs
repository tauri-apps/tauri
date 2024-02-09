use clap::{Parser, Subcommand};

use crate::Result;

mod add;
mod create;
mod ls;
mod rm;

#[derive(Debug, Parser)]
#[clap(about = "Manage or create permissions for your app or plugin")]
pub struct Cli {
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
  Create(create::Options),
  Add(add::Options),
  #[clap(alias = "remove")]
  Rm(rm::Options),
  #[clap(alias = "list")]
  Ls(ls::Options),
}

pub fn command(cli: Cli) -> Result<()> {
  match cli.command {
    Commands::Create(options) => create::command(options),
    Commands::Add(options) => add::command(options),
    Commands::Rm(options) => rm::command(options),
    Commands::Ls(options) => ls::command(options),
  }
}
