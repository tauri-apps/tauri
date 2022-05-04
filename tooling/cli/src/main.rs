// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::env::args_os;
use std::ffi::OsStr;
use std::io::Write;
use std::path::Path;
use std::process::exit;

use anyhow::Context;
use env_logger::fmt::Color;
use env_logger::{Builder, Env};
use log::{log_enabled, Level};

fn main() -> tauri_cli::Result<()> {
  let mut builder = Builder::from_default_env();
  builder
    .format_indent(Some(12))
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
    .parse_env(Env::new().default_filter_or("Info"))
    .init();

  let mut args = args_os().peekable();
  let bin_name = match args
    .next()
    .as_deref()
    .map(Path::new)
    .and_then(Path::file_stem)
    .and_then(OsStr::to_str)
  {
    Some("cargo-tauri") => {
      if args.peek().and_then(|s| s.to_str()) == Some("tauri") {
        // remove the extra cargo subcommand
        args.next();
        Some("cargo tauri".into())
      } else {
        Some("cargo-tauri".into())
      }
    }
    Some(stem) => Some(stem.to_string()),
    None => {
      eprintln!("cargo-tauri wrapper unable to read first argument");
      exit(1);
    }
  };

  tauri_cli::run(args, bin_name).context("Try running with --verbose to see command output")
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
