// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::OsString,
  path::{Path, PathBuf},
  process::Command,
};

use crate::{assert_command, CommandExt};
use anyhow::Result;
use rand::distributions::{Alphanumeric, DistString};

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
    let tmp_dir = tempfile::tempdir()?;
    let cert_path = tmp_dir.path().join("cert.p12");
    super::decode_base64(certificate_encoded, &cert_path)?;
    Self::with_certificate_file(&cert_path, certificate_password)
  }

  pub fn with_certificate_file(cert_path: &Path, certificate_password: &OsString) -> Result<Self> {
    let home_dir =
      dirs_next::home_dir().ok_or_else(|| anyhow::anyhow!("failed to resolve home dir"))?;
    let keychain_path = home_dir.join("Library").join("Keychains").join(format!(
      "{}.keychain-db",
      Alphanumeric.sample_string(&mut rand::thread_rng(), 16)
    ));
    let keychain_password = Alphanumeric.sample_string(&mut rand::thread_rng(), 16);

    let keychain_list_output = Command::new("security")
      .args(["list-keychain", "-d", "user"])
      .output()?;

    assert_command(
      Command::new("security")
        .args(["create-keychain", "-p", &keychain_password])
        .arg(&keychain_path)
        .piped(),
      "failed to create keychain",
    )?;

    assert_command(
      Command::new("security")
        .args(["unlock-keychain", "-p", &keychain_password])
        .arg(&keychain_path)
        .piped(),
      "failed to set unlock keychain",
    )?;

    assert_command(
      Command::new("security")
        .arg("import")
        .arg(cert_path)
        .arg("-P")
        .arg(certificate_password)
        .args([
          "-T",
          "/usr/bin/codesign",
          "-T",
          "/usr/bin/pkgbuild",
          "-T",
          "/usr/bin/productbuild",
        ])
        .arg("-k")
        .arg(&keychain_path)
        .piped(),
      "failed to import keychain certificate",
    )?;

    assert_command(
      Command::new("security")
        .args(["set-keychain-settings", "-t", "3600", "-u"])
        .arg(&keychain_path)
        .piped(),
      "failed to set keychain settings",
    )?;

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
    )?;

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
    )?;

    let signing_identity = identity::list(&keychain_path)
      .map(|l| l.first().cloned())?
      .ok_or_else(|| anyhow::anyhow!("failed to resolve signing identity"))?;

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
    println!("Signing with identity \"{}\"", identity);

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

    assert_command(codesign.piped(), "failed to sign app")?;

    Ok(())
  }

  pub fn path(&self) -> Option<&Path> {
    self.path.as_deref()
  }
}
