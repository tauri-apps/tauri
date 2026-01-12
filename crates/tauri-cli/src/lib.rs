// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This Rust executable provides the full interface to all of the required activities for which the CLI is required. It will run on macOS, Windows, and Linux.

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![cfg(any(target_os = "macos", target_os = "linux", windows))]

mod acl;
mod add;
mod build;
mod bundle;
mod completions;
mod dev;
mod error;
mod helpers;
mod icon;
mod info;
mod init;
mod inspect;
mod interface;
mod migrate;
mod mobile;
mod plugin;
mod remove;
mod signer;

use clap::{ArgAction, CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use env_logger::fmt::style::{AnsiColor, Style};
use env_logger::Builder;
pub use error::{Error, ErrorExt, Result};
use log::Level;
use serde::{Deserialize, Serialize};
use std::io::{BufReader, Write};
use std::process::{exit, Command, ExitStatus, Output, Stdio};
use std::{
  ffi::OsString,
  fmt::Display,
  fs::read_to_string,
  io::BufRead,
  path::PathBuf,
  str::FromStr,
  sync::{Arc, Mutex},
};

use crate::error::Context;

/// Tauri configuration argument option.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigValue(pub(crate) serde_json::Value);

impl FromStr for ConfigValue {
  type Err = Error;

  fn from_str(config: &str) -> std::result::Result<Self, Self::Err> {
    if config.starts_with('{') {
      Ok(Self(serde_json::from_str(config).with_context(|| {
        format!("failed to parse config `{config}` as JSON")
      })?))
    } else {
      let path = PathBuf::from(config);
      let raw =
        read_to_string(&path).fs_context("failed to read configuration file", path.clone())?;

      match path.extension().and_then(|ext| ext.to_str()) {
        Some("toml") => Ok(Self(::toml::from_str(&raw).with_context(|| {
          format!("failed to parse config at {} as TOML", path.display())
        })?)),
        Some("json5") => Ok(Self(::json5::from_str(&raw).with_context(|| {
          format!("failed to parse config at {} as JSON5", path.display())
        })?)),
        // treat all other extensions as json
        _ => Ok(Self(
          // from tauri-utils/src/config/parse.rs:
          // we also want to support **valid** json5 in the .json extension
          // if the json5 is not valid the serde_json error for regular json will be returned.
          match ::json5::from_str(&raw) {
            Ok(json5) => json5,
            Err(_) => serde_json::from_str(&raw)
              .with_context(|| format!("failed to parse config at {} as JSON", path.display()))?,
          },
        )),
      }
    }
  }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum RunMode {
  Desktop,
  #[cfg(target_os = "macos")]
  Ios,
  Android,
}

impl Display for RunMode {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        Self::Desktop => "desktop",
        #[cfg(target_os = "macos")]
        Self::Ios => "iOS",
        Self::Android => "android",
      }
    )
  }
}

#[derive(Deserialize)]
pub struct VersionMetadata {
  tauri: String,
  #[serde(rename = "tauri-build")]
  tauri_build: String,
  #[serde(rename = "tauri-plugin")]
  tauri_plugin: String,
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
pub(crate) struct Cli {
  /// Enables verbose logging
  #[clap(short, long, global = true, action = ArgAction::Count)]
  verbose: u8,
  #[clap(subcommand)]
  command: Commands,
}

#[derive(Subcommand)]
enum Commands {
  Init(init::Options),
  Dev(dev::Options),
  Build(build::Options),
  Bundle(bundle::Options),
  Android(mobile::android::Cli),
  #[cfg(target_os = "macos")]
  Ios(mobile::ios::Cli),
  /// Migrate from v1 to v2
  Migrate,
  Info(info::Options),
  Add(add::Options),
  Remove(remove::Options),
  Plugin(plugin::Cli),
  Icon(icon::Options),
  Signer(signer::Cli),
  Completions(completions::Options),
  Permission(acl::permission::Cli),
  Capability(acl::capability::Cli),
  Inspect(inspect::Cli),
}

fn format_error<I: CommandFactory>(err: clap::Error) -> clap::Error {
  let mut app = I::command();
  err.format(&mut app)
}

fn get_verbosity(cli_verbose: u8) -> u8 {
  std::env::var("TAURI_CLI_VERBOSITY")
    .ok()
    .and_then(|v| v.parse().ok())
    .unwrap_or(cli_verbose)
}

/// Run the Tauri CLI with the passed arguments, exiting if an error occurs.
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
pub fn run<I, A>(args: I, bin_name: Option<String>)
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  if let Err(e) = try_run(args, bin_name) {
    log::error!("{e}");
    exit(1);
  }
}

/// Run the Tauri CLI with the passed arguments.
///
/// It is similar to [`run`], but instead of exiting on an error, it returns a result.
pub fn try_run<I, A>(args: I, bin_name: Option<String>) -> Result<()>
where
  I: IntoIterator<Item = A>,
  A: Into<OsString> + Clone,
{
  let cli = match bin_name {
    Some(bin_name) => Cli::command().bin_name(bin_name),
    None => Cli::command(),
  };
  let cli_ = cli.clone();
  let matches = cli.get_matches_from(args);

  let res = Cli::from_arg_matches(&matches).map_err(format_error::<Cli>);
  let cli = match res {
    Ok(s) => s,
    Err(e) => e.exit(),
  };
  // set the verbosity level so subsequent CLI calls (xcode-script, android-studio-script) refer to it
  let verbosity_number = get_verbosity(cli.verbose);
  std::env::set_var("TAURI_CLI_VERBOSITY", verbosity_number.to_string());

  let mut builder = Builder::from_default_env();
  if let Err(err) = builder
    .format_indent(Some(12))
    .filter(None, verbosity_level(verbosity_number).to_level_filter())
    // golbin spams an insane amount of really technical logs on the debug level so we're reducing one level
    .filter(
      Some("goblin"),
      verbosity_level(verbosity_number.saturating_sub(1)).to_level_filter(),
    )
    // handlebars is not that spammy but its debug logs are typically far from being helpful
    .filter(
      Some("handlebars"),
      verbosity_level(verbosity_number.saturating_sub(1)).to_level_filter(),
    )
    .format(|f, record| {
      let mut is_command_output = false;
      if let Some(action) = record.key_values().get("action".into()) {
        let action = action.to_cow_str().unwrap();
        is_command_output = action == "stdout" || action == "stderr";
        if !is_command_output {
          let style = Style::new().fg_color(Some(AnsiColor::Green.into())).bold();
          write!(f, "{style}{action:>12}{style:#} ")?;
        }
      } else {
        let style = f.default_level_style(record.level()).bold();
        write!(
          f,
          "{style}{:>12}{style:#} ",
          prettyprint_level(record.level())
        )?;
      }

      if !is_command_output && log::log_enabled!(Level::Debug) {
        let style = Style::new().fg_color(Some(AnsiColor::Black.into()));
        write!(f, "[{style}{}{style:#}] ", record.target())?;
      }

      writeln!(f, "{}", record.args())
    })
    .try_init()
  {
    eprintln!("Failed to attach logger: {err}");
  }

  match cli.command {
    Commands::Build(options) => build::command(options, cli.verbose)?,
    Commands::Bundle(options) => bundle::command(options, cli.verbose)?,
    Commands::Dev(options) => dev::command(options)?,
    Commands::Add(options) => add::command(options)?,
    Commands::Remove(options) => remove::command(options)?,
    Commands::Icon(options) => icon::command(options)?,
    Commands::Info(options) => info::command(options)?,
    Commands::Init(options) => init::command(options)?,
    Commands::Plugin(cli) => plugin::command(cli)?,
    Commands::Signer(cli) => signer::command(cli)?,
    Commands::Completions(options) => completions::command(options, cli_)?,
    Commands::Permission(options) => acl::permission::command(options)?,
    Commands::Capability(options) => acl::capability::command(options)?,
    Commands::Android(c) => mobile::android::command(c, cli.verbose)?,
    #[cfg(target_os = "macos")]
    Commands::Ios(c) => mobile::ios::command(c, cli.verbose)?,
    Commands::Migrate => migrate::command()?,
    Commands::Inspect(cli) => inspect::command(cli)?,
  }

  Ok(())
}

/// This maps the occurrence of `--verbose` flags to the correct log level
fn verbosity_level(num: u8) -> Level {
  match num {
    0 => Level::Info,
    1 => Level::Debug,
    _ => Level::Trace,
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
  fn piped(&mut self) -> std::io::Result<ExitStatus>;
  fn output_ok(&mut self) -> crate::Result<Output>;
}

impl CommandExt for Command {
  fn piped(&mut self) -> std::io::Result<ExitStatus> {
    self.stdin(os_pipe::dup_stdin()?);
    self.stdout(os_pipe::dup_stdout()?);
    self.stderr(os_pipe::dup_stderr()?);

    let program = self.get_program().to_string_lossy().into_owned();
    let args = self
      .get_args()
      .map(|a| a.to_string_lossy())
      .collect::<Vec<_>>()
      .join(" ");

    log::debug!(action = "Running"; "Command `{program} {args}`");
    self.status()
  }

  fn output_ok(&mut self) -> crate::Result<Output> {
    let program = self.get_program().to_string_lossy().into_owned();
    let args = self
      .get_args()
      .map(|a| a.to_string_lossy())
      .collect::<Vec<_>>()
      .join(" ");
    let cmdline = format!("{program} {args}");
    log::debug!(action = "Running"; "Command `{cmdline}`");

    self.stdout(Stdio::piped());
    self.stderr(Stdio::piped());

    let mut child = self
      .spawn()
      .with_context(|| format!("failed to run command `{cmdline}`"))?;

    let mut stdout = child.stdout.take().map(BufReader::new).unwrap();
    let stdout_lines = Arc::new(Mutex::new(Vec::new()));
    let stdout_lines_ = stdout_lines.clone();
    std::thread::spawn(move || {
      let mut line = String::new();
      if let Ok(mut lines) = stdout_lines_.lock() {
        loop {
          line.clear();
          match stdout.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
              log::debug!(action = "stdout"; "{}", line.trim_end());
              lines.extend(line.as_bytes());
            }
            Err(_) => (),
          }
        }
      }
    });

    let mut stderr = child.stderr.take().map(BufReader::new).unwrap();
    let stderr_lines = Arc::new(Mutex::new(Vec::new()));
    let stderr_lines_ = stderr_lines.clone();
    std::thread::spawn(move || {
      let mut line = String::new();
      if let Ok(mut lines) = stderr_lines_.lock() {
        loop {
          line.clear();
          match stderr.read_line(&mut line) {
            Ok(0) => break,
            Ok(_) => {
              log::debug!(action = "stderr"; "{}", line.trim_end());
              lines.extend(line.as_bytes());
            }
            Err(_) => (),
          }
        }
      }
    });

    let status = child
      .wait()
      .with_context(|| format!("failed to run command `{cmdline}`"))?;

    let output = Output {
      status,
      stdout: std::mem::take(&mut *stdout_lines.lock().unwrap()),
      stderr: std::mem::take(&mut *stderr_lines.lock().unwrap()),
    };

    if output.status.success() {
      Ok(output)
    } else {
      crate::error::bail!(
        "failed to run command `{cmdline}`: command exited with status code {}",
        output.status.code().unwrap_or(-1)
      );
    }
  }
}

#[cfg(test)]
mod tests {
  use clap::CommandFactory;

  use crate::Cli;

  #[test]
  fn verify_cli() {
    Cli::command().debug_assert();
  }

  #[test]
  fn help_output_includes_build() {
    let help = Cli::command().render_help().to_string();
    assert!(help.contains("Build"));
  }
}
