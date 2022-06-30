use crate::Result;
use clap::{Command, Parser};
use clap_complete::{generate, Shell};
use log::info;
use std::io::Cursor;

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

fn print_completions(shell: Shell, cmd: &mut Command) {
  let bin_name = cmd
    .get_bin_name()
    .map(|s| s.to_string())
    .unwrap_or_else(|| cmd.get_name().to_string());
  let cmd_name = cmd.get_name().to_string().replace('-', "_");

  let mut buffer = Cursor::new(Vec::new());
  generate(shell, cmd, &cmd_name, &mut buffer);

  let b = buffer.into_inner();
  let completions = String::from_utf8_lossy(&b);

  let shell_completions = match shell {
    Shell::Bash => completions.replace(
      &format!("-o default {}", cmd_name),
      &format!("-o default {}", bin_name),
    ),
    Shell::Fish => {
      completions.replace(&format!("-c {}", cmd_name), &format!("-c \"{}\"", bin_name))
    }
    Shell::Zsh => completions.replace(
      &format!("compdef {}", cmd_name),
      &format!("compdef {}", bin_name),
    ),
    Shell::PowerShell => completions.replace(&format!("'{}", cmd_name), &format!("'{}", bin_name)),
    _ => completions.into_owned(),
  };

  print!("{}", shell_completions);
}

pub fn command(options: Options, cmd: &mut Command) -> Result<()> {
  info!("Generating completion file for {}...", options.shell);

  print_completions(options.shell, cmd);

  Ok(())
}
