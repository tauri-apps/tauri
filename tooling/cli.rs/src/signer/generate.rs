// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::updater_signature::{generate_key, save_keypair},
  Result,
};
use clap::ArgMatches;
use std::path::PathBuf;

pub struct GenerateOptions {
  password: Option<String>,
  output_path: Option<PathBuf>,
  force: bool,
}

impl From<&ArgMatches> for GenerateOptions {
  fn from(matches: &ArgMatches) -> Self {
    let password = matches.value_of("password");
    let no_password = matches.is_present("no-password");
    let write_keys = matches.value_of("write-keys");
    let force = matches.is_present("force");

    Self {
      password: if no_password {
        Some("".to_owned())
      } else {
        password.map(ToString::to_string)
      },
      output_path: write_keys.map(Into::into),
      force,
    }
  }
}

pub fn command(matches: &ArgMatches) -> Result<()> {
  let options = GenerateOptions::from(matches);
  let keypair = generate_key(options.password).expect("Failed to generate key");

  if let Some(output_path) = options.output_path {
    let (secret_path, public_path) =
      save_keypair(options.force, output_path, &keypair.sk, &keypair.pk)
        .expect("Unable to write keypair");

    println!(
        "\nYour keypair was generated successfully\nPrivate: {} (Keep it secret!)\nPublic: {}\n---------------------------",
        secret_path.display(),
        public_path.display()
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

  println!("\nEnvironment variabled used to sign:\n`TAURI_PRIVATE_KEY`  Path or String of your private key\n`TAURI_KEY_PASSWORD`  Your private key password (optional)\n\nATTENTION: If you lose your private key OR password, you'll not be able to sign your update package and updates will not works.\n---------------------------\n");

  Ok(())
}
