// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(not(any(target_os = "macos", target_os = "linux", windows)))]
fn main() {
  println!("The Tauri CLI is not supported on this platform");
  std::process::exit(1);
}

#[cfg(any(target_os = "macos", target_os = "linux", windows))]
fn main() {
  use std::env::args_os;
  use std::ffi::OsStr;
  use std::path::Path;
  use std::process::exit;

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

  tauri_cli::run(args, bin_name)
}
