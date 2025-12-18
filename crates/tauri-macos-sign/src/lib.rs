// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::{OsStr, OsString},
  path::{Path, PathBuf},
  process::{Command, ExitStatus},
};

use serde::Deserialize;

pub mod certificate;
mod keychain;
mod provisioning_profile;

pub use keychain::{Keychain, Team};
pub use provisioning_profile::ProvisioningProfile;

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("failed to create temp directory: {0}")]
  TempDir(std::io::Error),
  #[error("failed to resolve home dir")]
  ResolveHomeDir,
  #[error("failed to resolve signing identity")]
  ResolveSigningIdentity,
  #[error("failed to decode provisioning profile")]
  FailedToDecodeProvisioningProfile,
  #[error("could not find provisioning profile UUID")]
  FailedToFindProvisioningProfileUuid,
  #[error("{context} {path}: {error}")]
  Plist {
    context: &'static str,
    path: PathBuf,
    error: plist::Error,
  },
  #[error("failed to upload app to Apple's notarization servers: {error}")]
  FailedToUploadApp { error: std::io::Error },
  #[error("failed to notarize app: {0}")]
  Notarize(String),
  #[error("failed to parse notarytool output as JSON: {output}")]
  ParseNotarytoolOutput { output: String },
  #[error("failed to run command {command}: {error}")]
  CommandFailed {
    command: String,
    error: std::io::Error,
  },
  #[error("{context} {path}: {error}")]
  Fs {
    context: &'static str,
    path: PathBuf,
    error: std::io::Error,
  },
  #[error("failed to parse X509 certificate: {error}")]
  X509Certificate {
    error: x509_certificate::X509CertificateError,
  },
  #[error("failed to create PFX from self signed certificate")]
  FailedToCreatePFX,
  #[error("failed to create self signed certificate: {error}")]
  FailedToCreateSelfSignedCertificate {
    error: Box<apple_codesign::AppleCodesignError>,
  },
  #[error("failed to encode DER: {error}")]
  FailedToEncodeDER { error: std::io::Error },
  #[error("certificate missing common name")]
  CertificateMissingCommonName,
  #[error("certificate missing organization unit for common name {common_name}")]
  CertificateMissingOrganizationUnit { common_name: String },
}

pub type Result<T> = std::result::Result<T, Error>;

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

    self.status()
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
  #[serde(default)]
  status: Option<String>,
  message: String,
}

pub fn notarize(
  keychain: &Keychain,
  app_bundle_path: &Path,
  auth: &AppleNotarizationCredentials,
) -> Result<()> {
  notarize_inner(keychain, app_bundle_path, auth, true)
}

pub fn notarize_without_stapling(
  keychain: &Keychain,
  app_bundle_path: &Path,
  auth: &AppleNotarizationCredentials,
) -> Result<()> {
  notarize_inner(keychain, app_bundle_path, auth, false)
}

fn notarize_inner(
  keychain: &Keychain,
  app_bundle_path: &Path,
  auth: &AppleNotarizationCredentials,
  wait: bool,
) -> Result<()> {
  let bundle_stem = app_bundle_path
    .file_stem()
    .expect("failed to get bundle filename");

  let tmp_dir = tempfile::tempdir().map_err(Error::TempDir)?;
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
  )
  .map_err(|error| Error::CommandFailed {
    command: "ditto".to_string(),
    error,
  })?;

  // sign the zip file
  keychain.sign(&zip_path, None, false)?;

  let mut notarize_args = vec![
    "notarytool",
    "submit",
    zip_path
      .to_str()
      .expect("failed to convert zip_path to string"),
    "--output-format",
    "json",
  ];
  if wait {
    notarize_args.push("--wait");
  }
  let notarize_args = notarize_args;

  println!("Notarizing {}", app_bundle_path.display());

  let output = Command::new("xcrun")
    .args(notarize_args)
    .notarytool_args(auth, tmp_dir.path())?
    .output()
    .map_err(|error| Error::FailedToUploadApp { error })?;

  if !output.status.success() {
    return Err(Error::Notarize(
      String::from_utf8_lossy(&output.stderr).into_owned(),
    ));
  }

  let output_str = String::from_utf8_lossy(&output.stdout);
  if let Ok(submit_output) = serde_json::from_str::<NotarytoolSubmitOutput>(&output_str) {
    let log_message = format!(
      "{} with status {} for id {} ({})",
      if wait { "Finished" } else { "Submitted" },
      submit_output.status.as_deref().unwrap_or("Pending"),
      submit_output.id,
      submit_output.message
    );
    // status is empty when not waiting for the notarization to finish
    if submit_output.status.map_or(!wait, |s| s == "Accepted") {
      println!("Notarizing {log_message}");

      if wait {
        println!("Stapling app...");
        staple_app(app_bundle_path.to_path_buf())?;
      } else {
        println!("Not waiting for notarization to finish.");
        println!("You can use `xcrun notarytool log` to check the notarization progress.");
        println!(
          "When it's done you can optionally staple your app via `xcrun stapler staple {}`",
          app_bundle_path.display()
        );
      }

      Ok(())
    } else if let Ok(output) = Command::new("xcrun")
      .args(["notarytool", "log"])
      .arg(&submit_output.id)
      .notarytool_args(auth, tmp_dir.path())?
      .output()
    {
      Err(Error::Notarize(format!(
        "{log_message}\nLog:\n{}",
        String::from_utf8_lossy(&output.stdout)
      )))
    } else {
      Err(Error::Notarize(log_message))
    }
  } else {
    Err(Error::ParseNotarytoolOutput {
      output: output_str.into_owned(),
    })
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
    .map_err(|error| Error::CommandFailed {
      command: "xcrun stapler staple".to_string(),
      error,
    })?;

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
            std::fs::write(&key_path, k).map_err(|error| Error::Fs {
              context: "failed to write notarization API key to temp file",
              path: key_path.clone(),
              error,
            })?;
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
  let tmp_dir = tempfile::tempdir().map_err(Error::TempDir)?;

  let src_path = tmp_dir.path().join("src");
  let base64 = base64
    .to_str()
    .expect("failed to convert base64 to string")
    .as_bytes();

  // as base64 contain whitespace decoding may be broken
  // https://github.com/marshallpierce/rust-base64/issues/105
  // we'll use builtin base64 command from the OS
  std::fs::write(&src_path, base64).map_err(|error| Error::Fs {
    context: "failed to write base64 to temp file",
    path: src_path.clone(),
    error,
  })?;

  assert_command(
    std::process::Command::new("base64")
      .arg("--decode")
      .arg("-i")
      .arg(&src_path)
      .arg("-o")
      .arg(out_path)
      .piped(),
    "failed to decode certificate",
  )
  .map_err(|error| Error::CommandFailed {
    command: "base64 --decode".to_string(),
    error,
  })?;

  Ok(())
}

fn assert_command(
  response: std::result::Result<std::process::ExitStatus, std::io::Error>,
  error_message: &str,
) -> std::io::Result<()> {
  let status =
    response.map_err(|e| std::io::Error::new(e.kind(), format!("{error_message}: {e}")))?;
  if !status.success() {
    Err(std::io::Error::other(error_message))
  } else {
    Ok(())
  }
}
