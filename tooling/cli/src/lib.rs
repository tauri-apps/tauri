// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

pub use anyhow::Result;

mod build;
mod dev;
mod helpers;
mod info;
mod init;
mod interface;
mod plugin;
mod signer;

use clap::{FromArgMatches, IntoApp, Parser, Subcommand};
use env_logger::fmt::Color;
use env_logger::Builder;
use log::{debug, log_enabled, Level};
use serde::Deserialize;
use std::ffi::OsString;
use std::io::Write;
use std::process::{Command, Output};

#[derive(Deserialize)]
pub struct VersionMetadata {
  tauri: String,
  #[serde(rename = "tauri-build")]
  tauri_build: String,
}

#[derive(Deserialize)]
pub struct PackageJson {
  name: Option<String>,
  version: Option<String>,
  product_name: Option<String>,
}

#[derive(Parser)]
#[clap(
  author,
  version,
  about,
  bin_name("cargo-tauri"),
  subcommand_required(true),
  arg_required_else_help(true),
  propagate_version(true),
  no_binary_name(true)
)]
struct Cli {
  /// Enables verbose logging
  #[clap(short, long, global = true, parse(from_occurrences))]
  verbose: usize,
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Build(build::Options),
  Dev(dev::Options),
  Info(info::Options),
  Init(init::Options),
  Plugin(plugin::Cli),
  Signer(signer::Cli),
}

fn format_error<I: IntoApp>(err: clap::Error) -> clap::Error {
  let mut app = I::command();
  err.format(&mut app)
}

/// Run the Tauri CLI with the passed arguments.
///
/// The passed arguments should have the binary argument(s) stripped out before being passed.
///
/// e.g.
/// 1. `tauri-cli 1 2 3` -> `1 2 3`
/// 2. `cargo tauri 1 2 3` -> `1 2 3`
/// 3. `node tauri.js 1 2 3` -> `1 2 3`
///
/// The passed `bin_name` parameter should be how you want the help messages to display the command.
/// This defaults to `cargo-tauri`, but should be set to how the program was called, such as
/// `cargo tauri`.
pub fn run<I, A>(args: I, bin_name: Option<String>) -> Result<()>
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  let matches = match bin_name {
    Some(bin_name) => Cli::command().bin_name(bin_name),
    None => Cli::command(),
  }
  .get_matches_from(args);

  let res = Cli::from_arg_matches(&matches).map_err(format_error::<Cli>);
  let cli = match res {
    Ok(s) => s,
    Err(e) => e.exit(),
  };

  let mut builder = Builder::from_default_env();
  let init_res = builder
    .format_indent(Some(12))
    .filter(None, level_from_usize(cli.verbose).to_level_filter())
    .format(|f, record| {
      if let Some(action) = record.key_values().get("action".into()) {
        let mut action_style = f.style();
        action_style.set_color(Color::Green).set_bold(true);

        write!(f, "{:>12} ", action_style.value(action.to_str().unwrap()))?;
      } else {
        let mut level_style = f.default_level_style(record.level());
        level_style.set_bold(true);

        write!(
          f,
          "{:>12} ",
          level_style.value(prettyprint_level(record.level()))
        )?;
      }

      if log_enabled!(Level::Debug) {
        let mut target_style = f.style();
        target_style.set_color(Color::Black);

        write!(f, "[{}] ", target_style.value(record.target()))?;
      }

      writeln!(f, "{}", record.args())
    })
    .try_init();

  if let Err(err) = init_res {
    eprintln!("Failed to attach logger: {}", err);
  }

  match cli.command {
    Commands::Build(options) => build::command(options)?,
    Commands::Dev(options) => dev::command(options)?,
    Commands::Info(options) => info::command(options)?,
    Commands::Init(options) => init::command(options)?,
    Commands::Plugin(cli) => plugin::command(cli)?,
    Commands::Signer(cli) => signer::command(cli)?,
  }

  Ok(())
}

/// This maps the occurrence of `--verbose` flags to the correct log level
fn level_from_usize(num: usize) -> Level {
  match num {
    0 => Level::Info,
    1 => Level::Debug,
    2.. => Level::Trace,
    _ => panic!(),
  }
}

/// The default string representation for `Level` is all uppercaps which doesn't mix well with the other printed actions.
fn prettyprint_level(lvl: Level) -> &'static str {
  match lvl {
    Level::Error => "Error",
    Level::Warn => "Warn",
    Level::Info => "Info",
    Level::Debug => "Debug",
    Level::Trace => "Trace",
  }
}

pub trait CommandExt {
  // The `pipe` function sets the stdout and stderr to properly
  // show the command output in the Node.js wrapper.
  fn pipe(&mut self) -> Result<&mut Self>;
  fn output_ok(&mut self) -> crate::Result<Output>;
}

impl CommandExt for Command {
  fn pipe(&mut self) -> Result<&mut Self> {
    self.stdout(os_pipe::dup_stdout()?);
    self.stderr(os_pipe::dup_stderr()?);
    Ok(self)
  }

  fn output_ok(&mut self) -> crate::Result<Output> {
    debug!(action = "Running"; "Command `{} {}`", self.get_program().to_string_lossy(), self.get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{} {}", acc, arg)));

    let output = self.output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
      debug!("Stdout: {}", stdout);
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
      debug!("Stderr: {}", stderr);
    }

    if output.status.success() {
      Ok(output)
    } else {
      Err(anyhow::anyhow!(
        String::from_utf8_lossy(&output.stderr).to_string()
      ))
    }
  }
}
