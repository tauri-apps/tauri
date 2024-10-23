// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::Result;
use anyhow::Context;
use clap::{Command, Parser};
use clap_complete::{generate, Shell};

use std::{fs::write, path::PathBuf};

const PKG_MANAGERS: &[&str] = &["cargo", "pnpm", "npm", "yarn", "bun", "deno"];

#[derive(Debug, Clone, Parser)]
#[clap(about = "Generate Tauri CLI shell completions for Bash, Zsh, PowerShell or Fish")]
pub struct Options {
  /// Shell to generate a completion script for.
  #[clap(short, long, verbatim_doc_comment)]
  shell: Shell,
  /// Output file for the shell completions. By default the completions are printed to stdout.
  #[clap(short, long)]
  output: Option<PathBuf>,
}

fn completions_for(shell: Shell, manager: &'static str, cmd: Command) -> Vec<u8> {
  let tauri = cmd.name("tauri");
  let mut command = if manager == "npm" || manager == "bun" {
    Command::new(manager)
      .bin_name(manager)
      .subcommand(Command::new("run").subcommand(tauri))
  } else if manager == "deno" {
    Command::new(manager)
      .bin_name(manager)
      .subcommand(Command::new("task").subcommand(tauri))
  } else {
    Command::new(manager).bin_name(manager).subcommand(tauri)
  };

  let mut buf = Vec::new();
  generate(shell, &mut command, manager, &mut buf);
  buf
}

fn get_completions(shell: Shell, cmd: Command) -> Result<String> {
  let completions = if shell == Shell::Bash {
    let mut completions =
      String::from_utf8_lossy(&completions_for(shell, "cargo", cmd)).into_owned();
    for &manager in PKG_MANAGERS {
      completions.push_str(&format!(
        "complete -F _cargo -o bashdefault -o default {} tauri\n",
        if manager == "npm" {
          "npm run"
        } else if manager == "bun" {
          "bun run"
        } else if manager == "deno" {
          "deno task"
        } else {
          manager
        }
      ));
    }
    completions
  } else {
    let mut buffer = String::new();

    for (i, manager) in PKG_MANAGERS.iter().enumerate() {
      let buf = String::from_utf8_lossy(&completions_for(shell, manager, cmd.clone())).into_owned();

      let completions = match shell {
        Shell::PowerShell => {
          if i != 0 {
            // namespaces have already been imported
            buf
              .replace("using namespace System.Management.Automation.Language", "")
              .replace("using namespace System.Management.Automation", "")
          } else {
            buf
          }
        }
        _ => buf,
      };

      buffer.push_str(&completions);
      buffer.push('\n');
    }

    buffer
  };

  Ok(completions)
}

pub fn command(options: Options, cmd: Command) -> Result<()> {
  log::info!("Generating completion file for {}...", options.shell);

  let completions = get_completions(options.shell, cmd)?;
  if let Some(output) = options.output {
    write(output, completions).context("failed to write to output path")?;
  } else {
    print!("{completions}");
  }

  Ok(())
}
