// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::{OsStr, OsString},
  path::{Path, PathBuf},
  process::{Command, ExitStatus},
};

use anyhow::{Context, Result};
use serde::Deserialize;

pub mod certificate;
mod keychain;
mod provisioning_profile;

pub use keychain::{Keychain, Team};
pub use provisioning_profile::ProvisioningProfile;

trait CommandExt {
  // The `pipe` function sets the stdout and stderr to properly
  // show the command output in the Node.js wrapper.
  fn piped(&mut self) -> std::io::Result<ExitStatus>;
}

impl CommandExt for Command {
  fn piped(&mut self) -> std::io::Result<ExitStatus> {
    self.stdin(os_pipe::dup_stdin()?);
    self.stdout(os_pipe::dup_stdout()?);
    self.stderr(os_pipe::dup_stderr()?);
    let program = self.get_program().to_string_lossy().into_owned();
    log::debug!(action = "Running"; "Command `{} {}`", program, self.get_args().map(|arg| arg.to_string_lossy()).fold(String::new(), |acc, arg| format!("{acc} {arg}")));

    self.status().map_err(Into::into)
  }
}

pub enum ApiKey {
  Path(PathBuf),
  Raw(Vec<u8>),
}

pub enum AppleNotarizationCredentials {
  AppleId {
    apple_id: OsString,
    password: OsString,
    team_id: OsString,
  },
  ApiKey {
    issuer: OsString,
    key_id: OsString,
    key: ApiKey,
  },
}

#[derive(Deserialize)]
struct NotarytoolSubmitOutput {
  id: String,
  status: String,
  message: String,
}

pub fn notarize(
  keychain: &Keychain,
  app_bundle_path: &Path,
  auth: &AppleNotarizationCredentials,
) -> Result<()> {
  let bundle_stem = app_bundle_path
    .file_stem()
    .expect("failed to get bundle filename");

  let tmp_dir = tempfile::tempdir()?;
  let zip_path = tmp_dir
    .path()
    .join(format!("{}.zip", bundle_stem.to_string_lossy()));
  let zip_args = vec![
    "-c",
    "-k",
    "--keepParent",
    "--sequesterRsrc",
    app_bundle_path
      .to_str()
      .expect("failed to convert bundle_path to string"),
    zip_path
      .to_str()
      .expect("failed to convert zip_path to string"),
  ];

  // use ditto to create a PKZip almost identical to Finder
  // this remove almost 99% of false alarm in notarization
  assert_command(
    Command::new("ditto").args(zip_args).piped(),
    "failed to zip app with ditto",
  )?;

  // sign the zip file
  keychain.sign(&zip_path, None, false)?;

  let notarize_args = vec![
    "notarytool",
    "submit",
    zip_path
      .to_str()
      .expect("failed to convert zip_path to string"),
    "--wait",
    "--output-format",
    "json",
  ];

  println!("Notarizing {}", app_bundle_path.display());

  let output = Command::new("xcrun")
    .args(notarize_args)
    .notarytool_args(auth, tmp_dir.path())?
    .output()
    .context("failed to upload app to Apple's notarization servers.")?;

  if !output.status.success() {
    return Err(
      anyhow::anyhow!("failed to notarize app")
        .context(String::from_utf8_lossy(&output.stderr).into_owned()),
    );
  }

  let output_str = String::from_utf8_lossy(&output.stdout);
  if let Ok(submit_output) = serde_json::from_str::<NotarytoolSubmitOutput>(&output_str) {
    let log_message = format!(
      "Finished with status {} for id {} ({})",
      submit_output.status, submit_output.id, submit_output.message
    );
    if submit_output.status == "Accepted" {
      println!("Notarizing {}", log_message);
      staple_app(app_bundle_path.to_path_buf())?;
      Ok(())
    } else if let Ok(output) = Command::new("xcrun")
      .args(["notarytool", "log"])
      .arg(&submit_output.id)
      .notarytool_args(auth, tmp_dir.path())?
      .output()
    {
      Err(anyhow::anyhow!(
        "{log_message}\nLog:\n{}",
        String::from_utf8_lossy(&output.stdout)
      ))
    } else {
      Err(anyhow::anyhow!("{log_message}"))
    }
  } else {
    Err(anyhow::anyhow!(
      "failed to parse notarytool output as JSON: `{output_str}`"
    ))
  }
}

fn staple_app(mut app_bundle_path: PathBuf) -> Result<()> {
  let app_bundle_path_clone = app_bundle_path.clone();
  let filename = app_bundle_path_clone
    .file_name()
    .expect("failed to get bundle filename")
    .to_str()
    .expect("failed to convert bundle filename to string");

  app_bundle_path.pop();

  Command::new("xcrun")
    .args(vec!["stapler", "staple", "-v", filename])
    .current_dir(app_bundle_path)
    .output()
    .context("failed to staple app.")?;

  Ok(())
}

pub trait NotarytoolCmdExt {
  fn notarytool_args(
    &mut self,
    auth: &AppleNotarizationCredentials,
    temp_dir: &Path,
  ) -> Result<&mut Self>;
}

impl NotarytoolCmdExt for Command {
  fn notarytool_args(
    &mut self,
    auth: &AppleNotarizationCredentials,
    temp_dir: &Path,
  ) -> Result<&mut Self> {
    match auth {
      AppleNotarizationCredentials::AppleId {
        apple_id,
        password,
        team_id,
      } => Ok(
        self
          .arg("--apple-id")
          .arg(apple_id)
          .arg("--password")
          .arg(password)
          .arg("--team-id")
          .arg(team_id),
      ),
      AppleNotarizationCredentials::ApiKey {
        key,
        key_id,
        issuer,
      } => {
        let key_path = match key {
          ApiKey::Raw(k) => {
            let key_path = temp_dir.join("AuthKey.p8");
            std::fs::write(&key_path, k)?;
            key_path
          }
          ApiKey::Path(p) => p.to_owned(),
        };

        Ok(
          self
            .arg("--key-id")
            .arg(key_id)
            .arg("--key")
            .arg(key_path)
            .arg("--issuer")
            .arg(issuer),
        )
      }
    }
  }
}

fn decode_base64(base64: &OsStr, out_path: &Path) -> Result<()> {
  let tmp_dir = tempfile::tempdir()?;

  let src_path = tmp_dir.path().join("src");
  let base64 = base64
    .to_str()
    .expect("failed to convert base64 to string")
    .as_bytes();

  // as base64 contain whitespace decoding may be broken
  // https://github.com/marshallpierce/rust-base64/issues/105
  // we'll use builtin base64 command from the OS
  std::fs::write(&src_path, base64)?;

  assert_command(
    std::process::Command::new("base64")
      .arg("--decode")
      .arg("-i")
      .arg(&src_path)
      .arg("-o")
      .arg(out_path)
      .piped(),
    "failed to decode certificate",
  )?;

  Ok(())
}

fn assert_command(
  response: Result<std::process::ExitStatus, std::io::Error>,
  error_message: &str,
) -> std::io::Result<()> {
  let status =
    response.map_err(|e| std::io::Error::new(e.kind(), format!("{error_message}: {e}")))?;
  if !status.success() {
    Err(std::io::Error::new(
      std::io::ErrorKind::Other,
      error_message,
    ))
  } else {
    Ok(())
  }
}
