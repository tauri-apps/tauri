// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::updater_signature::{generate_key, save_keypair},
  Result,
};
use clap::Parser;
use std::path::PathBuf;
use tauri_utils::display_path;

#[derive(Debug, Parser)]
#[clap(about = "Generate a new signing key to sign files")]
pub struct Options {
  /// Set private key password when signing
  #[clap(short, long)]
  password: Option<String>,
  /// Write private key to a file
  #[clap(short, long)]
  write_keys: Option<PathBuf>,
  /// Overwrite private key even if it exists on the specified path
  #[clap(short, long)]
  force: bool,
  /// Skip prompting for values
  #[clap(long, env = "CI")]
  ci: bool,
}

pub fn command(mut options: Options) -> Result<()> {
  if options.ci && options.password.is_none() {
    log::warn!("Generating new private key without password. For security reasons, we recommend setting a password instead.");
    options.password.replace("".into());
  }
  let keypair = generate_key(options.password).expect("Failed to generate key");

  if let Some(output_path) = options.write_keys {
    let (secret_path, public_path) =
      save_keypair(options.force, output_path, &keypair.sk, &keypair.pk)
        .expect("Unable to write keypair");

    println!();
    println!("Your keypair was generated successfully:");
    println!("Private: {} (Keep it secret!)", display_path(secret_path));
    println!("Public: {}", display_path(public_path));
    println!("---------------------------")
  } else {
    println!();
    println!("Your keys were generated successfully!",);
    println!();
    println!("Private: (Keep it secret!)");
    println!("{}", keypair.sk);
    println!();
    println!("Public:");
    println!("{}", keypair.pk);
  }

  println!();
  println!("Environment variables used to sign:");
  println!("- `TAURI_SIGNING_PRIVATE_KEY`: String of your private key");
  println!("- `TAURI_SIGNING_PRIVATE_KEY_PATH`: Path to your private key file");
  println!("- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`:  Your private key password (optional if key has no password)");
  println!();
  println!("ATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not work");

  Ok(())
}
