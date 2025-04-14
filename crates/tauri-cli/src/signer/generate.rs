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

    println!(
        "\nYour keypair was generated successfully\nPrivate: {} (Keep it secret!)\nPublic: {}\n---------------------------",
        display_path(secret_path),
        display_path(public_path)
        )
  } else {
    println!(
      "\nYour secret key was generated successfully - Keep it secret!\n{}\n\n",
      keypair.sk
    );
    println!(
          "Your public key was generated successfully:\n{}\n\nAdd the public key in your tauri.conf.json\n---------------------------\n",
          keypair.pk
        );
  }

  println!("\nEnvironment variables used to sign:");
  println!("`TAURI_SIGNING_PRIVATE_KEY`  Path or String of your private key");
  println!("`TAURI_SIGNING_PRIVATE_KEY_PASSWORD`  Your private key password (optional)");
  println!("\nATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not work.\n---------------------------\n");

  Ok(())
}
