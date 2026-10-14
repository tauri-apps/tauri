// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::{OsStr, OsString},
  io::Write,
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

use crate::{CommandExt, Error, Result, assert_command};
use rand::distr::{Alphanumeric, SampleString};

mod identity;

pub use identity::Team;

pub enum SigningIdentity {
  Team(Team),
  Identifier(String),
}

pub struct Keychain {
  // none means the default keychain must be used
  path: Option<PathBuf>,
  signing_identity: SigningIdentity,
}

impl Drop for Keychain {
  fn drop(&mut self) {
    if let Some(path) = &self.path {
      let _ = Command::new("security")
        .arg("delete-keychain")
        .arg(path)
        .piped();
    }
  }
}

impl Keychain {
  /// Use a certificate in the default keychain.
  pub fn with_signing_identity(identity: impl Into<String>) -> Self {
    Self {
      path: None,
      signing_identity: SigningIdentity::Identifier(identity.into()),
    }
  }

  /// Import certificate from base64 string.
  /// certificate_encoded is the p12 certificate base64 encoded.
  /// By example you can use; openssl base64 -in MyCertificate.p12 -out MyCertificate-base64.txt
  /// Then use the value of the base64 as `certificate_encoded`.
  /// You need to set certificate_password to the password you set when you exported your certificate.
  /// <https://help.apple.com/xcode/mac/current/#/dev154b28f09> see: `Export a signing certificate`
  pub fn with_certificate(
    certificate_encoded: &OsString,
    certificate_password: &OsString,
  ) -> Result<Self> {
    let tmp_dir = tempfile::tempdir().map_err(Error::TempDir)?;
    let cert_path = tmp_dir.path().join("cert.p12");
    super::decode_base64(certificate_encoded, &cert_path)?;
    Self::with_certificate_file(&cert_path, certificate_password)
  }

  pub fn with_certificate_file(cert_path: &Path, certificate_password: &OsString) -> Result<Self> {
    let home_dir = dirs::home_dir().ok_or(Error::ResolveHomeDir)?;
    let keychain_path = home_dir.join("Library").join("Keychains").join(format!(
      "{}.keychain-db",
      Alphanumeric.sample_string(&mut rand::rng(), 16)
    ));
    let keychain_password = Alphanumeric.sample_string(&mut rand::rng(), 16);

    let keychain_list_output = Command::new("security")
      .args(["list-keychain", "-d", "user"])
      .output()
      .map_err(|e| Error::CommandFailed {
        command: "security list-keychain -d user".to_string(),
        error: e,
      })?;

    assert_command(
      Command::new("security")
        .args(["create-keychain", "-p", &keychain_password])
        .arg(&keychain_path)
        .piped(),
      "failed to create keychain",
    )
    .map_err(|error| Error::CommandFailed {
      command: "security create-Keychain".to_string(),
      error,
    })?;

    assert_command(
      Command::new("security")
        .args(["unlock-keychain", "-p", &keychain_password])
        .arg(&keychain_path)
        .piped(),
      "failed to set unlock keychain",
    )
    .map_err(|error| Error::CommandFailed {
      command: "security unlock-keychain".to_string(),
      error,
    })?;

    import_certificate(cert_path, certificate_password, &keychain_path)?;

    assert_command(
      Command::new("security")
        .args(["set-keychain-settings", "-t", "3600", "-u"])
        .arg(&keychain_path)
        .piped(),
      "failed to set keychain settings",
    )
    .map_err(|error| Error::CommandFailed {
      command: "security set-keychain-settings".to_string(),
      error,
    })?;

    assert_command(
      Command::new("security")
        .args([
          "set-key-partition-list",
          "-S",
          "apple-tool:,apple:,codesign:",
          "-s",
          "-k",
          &keychain_password,
        ])
        .arg(&keychain_path)
        .piped(),
      "failed to set keychain settings",
    )
    .map_err(|error| Error::CommandFailed {
      command: "security set-key-partition-list".to_string(),
      error,
    })?;

    let current_keychains = String::from_utf8_lossy(&keychain_list_output.stdout)
      .split('\n')
      .map(|line| {
        line
          .trim_matches(|c: char| c.is_whitespace() || c == '"')
          .to_string()
      })
      .filter(|l| !l.is_empty())
      .collect::<Vec<String>>();

    assert_command(
      Command::new("security")
        .args(["list-keychain", "-d", "user", "-s"])
        .args(current_keychains)
        .arg(&keychain_path)
        .piped(),
      "failed to list keychain",
    )
    .map_err(|error| Error::CommandFailed {
      command: "security list-keychain".to_string(),
      error,
    })?;

    let signing_identity = identity::list(&keychain_path)
      .map(|l| l.first().cloned())?
      .ok_or(Error::ResolveSigningIdentity)?;

    Ok(Self {
      path: Some(keychain_path),
      signing_identity: SigningIdentity::Team(signing_identity),
    })
  }

  pub fn signing_identity(&self) -> String {
    match &self.signing_identity {
      SigningIdentity::Team(t) => t.certificate_name(),
      SigningIdentity::Identifier(i) => i.to_string(),
    }
  }

  pub fn team_id(&self) -> Option<&str> {
    match &self.signing_identity {
      SigningIdentity::Team(t) => Some(&t.id),
      SigningIdentity::Identifier(_) => None,
    }
  }

  pub fn sign(
    &self,
    path: &Path,
    entitlements_path: Option<&Path>,
    hardened_runtime: bool,
  ) -> Result<()> {
    let identity = match &self.signing_identity {
      SigningIdentity::Team(t) => t.certificate_name(),
      SigningIdentity::Identifier(i) => i.clone(),
    };
    println!("Signing with identity \"{identity}\"");

    println!("Signing {}", path.display());

    let mut args = vec!["--force", "-s", &identity];

    if hardened_runtime {
      args.push("--options");
      args.push("runtime");
    }

    let mut codesign = Command::new("codesign");
    codesign.args(args);
    if let Some(p) = &self.path {
      codesign.arg("--keychain").arg(p);
    }

    if let Some(entitlements_path) = entitlements_path {
      codesign.arg("--entitlements");
      codesign.arg(entitlements_path);
    }

    codesign.arg(path);

    assert_command(codesign.piped(), "failed to sign app").map_err(|error| {
      Error::CommandFailed {
        command: "codesign".to_string(),
        error,
      }
    })?;

    Ok(())
  }

  pub fn path(&self) -> Option<&Path> {
    self.path.as_deref()
  }
}

/// Imports the certificate and its private key into the given keychain.
///
/// The command is written to the stdin of `security -i` instead of being passed
/// as arguments, so the certificate password does not show up in the process list.
fn import_certificate(
  cert_path: &Path,
  certificate_password: &OsString,
  keychain_path: &Path,
) -> Result<()> {
  let command_failed = |error| Error::CommandFailed {
    command: "security import".to_string(),
    error,
  };

  let mut command_line = b"import".to_vec();
  for arg in [
    cert_path.as_os_str(),
    OsStr::new("-P"),
    certificate_password,
  ]
  .into_iter()
  .chain(
    [
      "-T",
      "/usr/bin/codesign",
      "-T",
      "/usr/bin/pkgbuild",
      "-T",
      "/usr/bin/productbuild",
      "-k",
    ]
    .map(OsStr::new),
  )
  .chain([keychain_path.as_os_str()])
  {
    command_line.push(b' ');
    command_line.extend(quote_interactive_arg(arg).map_err(command_failed)?);
  }
  command_line.push(b'\n');

  log::debug!(action = "Running"; "Command `security -i` with `security import` on stdin");
  let mut child = Command::new("security")
    .arg("-i")
    .stdin(Stdio::piped())
    .stdout(os_pipe::dup_stdout().map_err(command_failed)?)
    .stderr(os_pipe::dup_stderr().map_err(command_failed)?)
    .spawn()
    .map_err(command_failed)?;
  let write_result = child
    .stdin
    .take()
    .expect("stdin is piped")
    .write_all(&command_line);
  let status = child.wait();
  write_result.map_err(command_failed)?;

  assert_command(status, "failed to import keychain certificate").map_err(command_failed)
}

/// Quotes an argument for the `security -i` command parser,
/// which splits on whitespace and supports `"` quoting with `\` escapes.
fn quote_interactive_arg(arg: &OsStr) -> std::io::Result<Vec<u8>> {
  let bytes = arg.as_encoded_bytes();
  if bytes.iter().any(|b| matches!(b, b'\n' | b'\r')) {
    return Err(std::io::Error::new(
      std::io::ErrorKind::InvalidInput,
      "certificate path, password and keychain path must not contain line breaks",
    ));
  }

  let mut quoted = Vec::with_capacity(bytes.len() + 2);
  quoted.push(b'"');
  for &b in bytes {
    if matches!(b, b'"' | b'\\') {
      quoted.push(b'\\');
    }
    quoted.push(b);
  }
  quoted.push(b'"');
  Ok(quoted)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn quotes_interactive_args() {
    assert_eq!(
      quote_interactive_arg(OsStr::new(r#"p a"s\s'x"#)).unwrap(),
      br#""p a\"s\\s'x""#
    );
    assert_eq!(quote_interactive_arg(OsStr::new("")).unwrap(), b"\"\"");
    assert!(quote_interactive_arg(OsStr::new("a\nimport")).is_err());
  }

  #[cfg(target_os = "macos")]
  mod import {
    use super::super::*;
    use crate::certificate::{
      CertificateProfile, SelfSignedCertificateRequest, generate_self_signed,
    };

    /// Temporary keychain that is not added to the user's search list.
    struct TempKeychain {
      dir: tempfile::TempDir,
      path: PathBuf,
    }

    impl TempKeychain {
      fn new() -> Self {
        // includes a space to exercise quoting
        let dir = tempfile::Builder::new()
          .prefix("tauri sign")
          .tempdir()
          .unwrap();
        let path = dir.path().join("test.keychain-db");
        let status = Command::new("security")
          .args(["create-keychain", "-p", "test"])
          .arg(&path)
          .status()
          .unwrap();
        assert!(status.success(), "failed to create test keychain");
        Self { dir, path }
      }
    }

    impl Drop for TempKeychain {
      fn drop(&mut self) {
        let _ = Command::new("security")
          .arg("delete-keychain")
          .arg(&self.path)
          .status();
      }
    }

    fn import(password: &str, import_password: &str) -> Result<TempKeychain> {
      let keychain = TempKeychain::new();
      let cert_path = keychain.dir.path().join("cert.p12");
      let p12 = generate_self_signed(SelfSignedCertificateRequest {
        algorithm: "rsa".into(),
        profile: CertificateProfile::AppleDevelopment,
        team_id: "TAURITEST".into(),
        person_name: "Tauri Test".into(),
        country_name: "US".into(),
        validity_days: 1,
        password: password.into(),
      })?;
      std::fs::write(&cert_path, p12).unwrap();
      import_certificate(&cert_path, &import_password.into(), &keychain.path)?;
      Ok(keychain)
    }

    #[test]
    fn imports_with_password_needing_quotes() {
      let password = r#"p a"s\s'x -k /tmp/other"#;
      let keychain = import(password, password).unwrap();
      let output = Command::new("security")
        .args(["find-certificate", "-c", "Tauri Test"])
        .arg(&keychain.path)
        .output()
        .unwrap();
      assert!(output.status.success(), "certificate was not imported");
    }

    #[test]
    fn wrong_password_fails() {
      assert!(import("right", "wrong").is_err());
    }
  }
}
