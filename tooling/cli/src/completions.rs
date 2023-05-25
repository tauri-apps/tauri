// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use clap::{Command, Parser};
use clap_complete::{generate, Shell};
use log::info;
use std::io::Cursor;

const PKG_MANAGERS: &[&str] = &["cargo", "pnpm", "npm", "yarn"];

#[derive(Debug, Clone, Parser)]
#[clap(about = "Shell completions")]
pub struct Options {
  /// Shell to generate a completion script for.
  #[clap(short, long, verbatim_doc_comment)]
  shell: Shell,
}

fn commands_for_completions(shell: Shell, cmd: Command) -> Vec<Command> {
  if let Shell::Zsh = shell {
    let tauri = cmd.name("tauri");
    PKG_MANAGERS
      .iter()
      .map(|manager| Command::new(manager).subcommand(tauri.clone()))
      .collect()
  } else {
    vec![cmd]
  }
}

fn print_completions(shell: Shell, cmd: Command) {
  let bin_name = cmd
    .get_bin_name()
    .map(|s| s.to_string())
    .unwrap_or_else(|| cmd.get_name().to_string());
  let cmd_name = cmd.get_name().to_string().replace('-', "_");

  let mut buffer = Cursor::new(Vec::new());
  for mut cmd in commands_for_completions(shell, cmd) {
    let bin_name = cmd
      .get_bin_name()
      .map(|s| s.to_string())
      .unwrap_or_else(|| cmd.get_name().to_string());
    generate(shell, &mut cmd, bin_name, &mut buffer);
  }

  let b = buffer.into_inner();
  let completions = String::from_utf8_lossy(&b);

  let shell_completions = match shell {
    Shell::Bash => completions
      .replace(
        &format!("-o default {}", cmd_name),
        &format!("-o default {}", bin_name),
      )
      .replace(&cmd_name.replace('_', "__"), &cmd_name),
    Shell::Fish => {
      completions.replace(&format!("-c {}", cmd_name), &format!("-c \"{}\"", bin_name))
    }
    Shell::PowerShell => completions.replace(&format!("'{}", cmd_name), &format!("'{}", bin_name)),
    _ => completions.into_owned(),
  };

  print!("{}", shell_completions);

  for manager in PKG_MANAGERS {
    match shell {
      Shell::Bash => println!(
        "complete -F _{} -o bashdefault -o default {} tauri",
        cmd_name, manager
      ),
      Shell::Fish => {}
      Shell::Zsh => {}
      Shell::PowerShell => {}
      _ => {}
    };
  }
}

pub fn command(options: Options, cmd: Command) -> Result<()> {
  info!("Generating completion file for {}...", options.shell);

  print_completions(options.shell, cmd);

  Ok(())
}
