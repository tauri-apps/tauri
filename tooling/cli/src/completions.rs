use crate::{Cli, Result};
use clap::{Command, IntoApp, Parser};
use clap_complete::{generate, Generator, Shell};
use log::info;
use std::io;

#[derive(Debug, Clone, Parser)]
#[clap(about = "Shell completions")]
pub struct Options {
  /// Shell to generate a completion script for. 
  /// 
  /// Can be one of the following:
  /// - bash
  /// - elvish
  /// - fish
  /// - zsh
  /// - powershell
  #[clap(short, long, verbatim_doc_comment)]
  shell: Shell,
}

fn print_completions<G: Generator>(gen: G, cmd: &mut Command) {
  generate(gen, cmd, cmd.get_name().to_string(), &mut io::stdout());
}

pub fn command(options: Options) -> Result<()> {
  info!("Generating completion file for {}...", options.shell);

  let mut cmd = Cli::command();

  print_completions(options.shell, &mut cmd);

  Ok(())
}
