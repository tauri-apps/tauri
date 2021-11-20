// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use crate::{
  helpers::updater_signature::{read_key_from_file, sign_file},
  Result,
};
use anyhow::Context;
use clap::ArgMatches;

pub struct SignOptions {
  private_key: Option<String>,
  password: Option<String>,
  file: Option<PathBuf>,
}

impl From<&ArgMatches> for SignOptions {
  fn from(matches: &ArgMatches) -> Self {
    let private_key = matches.value_of("private-key");
    let private_key_path = matches.value_of("private-key-path");
    let file = matches.value_of("sign-file");
    let password = matches.value_of("password");
    let no_password = matches.is_present("no-password");

    Self {
      password: if no_password {
        Some("".to_owned())
      } else {
        password.map(ToString::to_string)
      },
      private_key: if let Some(private_key) = private_key_path {
        Some(read_key_from_file(Path::new(private_key)).expect("Unable to extract private key"))
      } else {
        private_key.map(ToString::to_string)
      },
      file: file.map(Into::into),
    }
  }
}

pub fn command(matches: &ArgMatches) -> Result<()> {
  let options = SignOptions::from(matches);
  if options.private_key.is_none() {
    return Err(anyhow::anyhow!(
      "Key generation aborted: Unable to find the private key".to_string(),
    ));
  }

  if options.password.is_none() {
    return Err(anyhow::anyhow!(
              "Please use --no-password to set empty password or add --password <password> if your private key have a password.".to_string(),
            ));
  }

  let (manifest_dir, signature) = sign_file(
    options.private_key.unwrap(),
    options.password.unwrap(),
    options.file.unwrap(),
  )
  .with_context(|| "failed to sign file")?;

  println!(
           "\nYour file was signed successfully, You can find the signature here:\n{}\n\nPublic signature:\n{}\n\nMake sure to include this into the signature field of your update server.",
           manifest_dir.display(),
           signature
         );

  Ok(())
}
