// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use crate::{
  error::Context,
  helpers::updater_signature::{secret_key, sign_file},
  Result,
};
use base64::Engine;
use clap::Parser;
use tauri_utils::display_path;

#[derive(Debug, Parser)]
#[clap(about = "Sign a file")]
pub struct Options {
  /// Load the private key from a string
  #[clap(
    short = 'k',
    long,
    conflicts_with("private_key_path"),
    env = "TAURI_SIGNING_PRIVATE_KEY"
  )]
  private_key: Option<String>,
  /// Load the private key from a file
  #[clap(
    short = 'f',
    long,
    conflicts_with("private_key"),
    env = "TAURI_SIGNING_PRIVATE_KEY_PATH"
  )]
  private_key_path: Option<PathBuf>,
  /// Set private key password when signing
  #[clap(short, long, env = "TAURI_SIGNING_PRIVATE_KEY_PASSWORD")]
  password: Option<String>,
  /// Sign the specified file
  file: PathBuf,
}

// Backwards compatibility with old env vars
// TODO: remove in v3.0
fn backward_env_vars(mut options: Options) -> Options {
  let get_env = |old, new| {
    if let Ok(old_value) = std::env::var(old) {
      println!(
      "\x1b[33mWarning: The environment variable '{old}' is deprecated. Please use '{new}' instead.\x1b[0m",
    );
      Some(old_value)
    } else {
      None
    }
  };

  options.private_key = options
    .private_key
    .or_else(|| get_env("TAURI_PRIVATE_KEY", "TAURI_SIGNING_PRIVATE_KEY"));

  options.private_key_path = options.private_key_path.or_else(|| {
    get_env("TAURI_PRIVATE_KEY_PATH", "TAURI_SIGNING_PRIVATE_KEY_PATH").map(PathBuf::from)
  });

  options.password = options.password.or_else(|| {
    get_env(
      "TAURI_PRIVATE_KEY_PASSWORD",
      "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
    )
  });
  options
}

pub fn command(mut options: Options) -> Result<()> {
  options = backward_env_vars(options);

  options.private_key = if let Some(private_key) = options.private_key_path {
    Some(std::fs::read_to_string(Path::new(&private_key)).expect("Unable to extract private key"))
  } else {
    options.private_key
  };
  let private_key = if let Some(pk) = options.private_key {
    pk
  } else {
    crate::error::bail!("Key generation aborted: Unable to find the private key");
  };

  if options.password.is_none() {
    println!("Signing without password.");
  }

  let (manifest_dir, signature) =
    sign_file(&secret_key(private_key, options.password)?, options.file)
      .with_context(|| "failed to sign file")?;

  println!(
           "\nYour file was signed successfully, You can find the signature here:\n{}\n\nPublic signature:\n{}\n\nMake sure to include this into the signature field of your update server.",
           display_path(manifest_dir),
           base64::engine::general_purpose::STANDARD.encode(signature.to_string())
         );

  Ok(())
}
