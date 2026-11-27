// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::OsString,
  path::{Path, PathBuf},
  process::Command,
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

    // owns the keychain from here on so it is deleted via the Drop impl above if
    // any of the remaining setup steps fail; the signing identity is resolved last
    let mut keychain = Self {
      path: Some(keychain_path.clone()),
      signing_identity: SigningIdentity::Identifier(String::new()),
    };

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

    keychain.signing_identity = SigningIdentity::Team(signing_identity);

    Ok(keychain)
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

/// Lists the code signing identities (certificates with a private key) in the user's keychains.
#[cfg(target_os = "macos")]
pub fn find_identities() -> Result<Vec<crate::certificate::CertificateInfo>> {
  Ok(
    keychain_identities()?
      .into_iter()
      .map(|(info, _)| info)
      .collect(),
  )
}

#[cfg(target_os = "macos")]
fn keychain_identities() -> Result<
  Vec<(
    crate::certificate::CertificateInfo,
    security_framework::identity::SecIdentity,
  )>,
> {
  use security_framework::item::{ItemClass, ItemSearchOptions, Limit, Reference, SearchResult};

  let results = match ItemSearchOptions::new()
    .class(ItemClass::identity())
    .load_refs(true)
    .limit(Limit::All)
    .search()
  {
    Ok(results) => results,
    // errSecItemNotFound
    Err(e) if e.code() == -25300 => return Ok(Vec::new()),
    Err(e) => return Err(Error::KeychainExport(e.to_string())),
  };

  let mut identities = Vec::new();
  for result in results {
    let SearchResult::Ref(Reference::Identity(identity)) = result else {
      continue;
    };
    let Ok(certificate) = identity.certificate() else {
      continue;
    };
    let Ok(certificate) = crate::certificate::parse_certificate(&certificate.to_der()) else {
      continue;
    };
    let info = crate::certificate::CertificateInfo::from_certificate(&certificate)?;
    let is_code_signing = info.common_name.as_deref().is_some_and(|name| {
      identity::CERTIFICATE_PREFIXES
        .iter()
        .any(|prefix| name.starts_with(prefix))
    });
    if is_code_signing {
      identities.push((info, identity));
    }
  }

  identities.sort_by(|(a, _), (b, _)| a.common_name.cmp(&b.common_name));
  identities.dedup_by(|(a, _), (b, _)| a.sha1_thumbprint == b.sha1_thumbprint);

  Ok(identities)
}

/// Exports a code signing identity from the user's keychains as a PKCS#12 archive.
///
/// The identity is matched by its SHA-1 thumbprint or by its full common name.
/// macOS asks the user to allow exporting the private key.
#[cfg(target_os = "macos")]
pub fn export_identity(
  identity: &str,
  password: &str,
) -> Result<(crate::certificate::CertificateInfo, Vec<u8>)> {
  use core_foundation::{base::TCFType, data::CFData, string::CFString};
  use security_framework_sys::import_export::{
    SEC_KEY_IMPORT_EXPORT_PARAMS_VERSION, SecItemExport, SecItemImportExportKeyParameters,
  };

  // not exposed by the security-framework-sys version we depend on
  #[allow(non_upper_case_globals)]
  const kSecFormatPKCS12: u32 = 12;

  let (info, sec_identity) = keychain_identities()?
    .into_iter()
    .find(|(info, _)| {
      info.sha1_thumbprint.eq_ignore_ascii_case(identity)
        || info.common_name.as_deref() == Some(identity)
    })
    .ok_or_else(|| Error::IdentityNotFound(identity.to_string()))?;

  let passphrase = CFString::new(password);
  let alert_title = CFString::new("Export signing certificate");
  let params = SecItemImportExportKeyParameters {
    version: SEC_KEY_IMPORT_EXPORT_PARAMS_VERSION,
    flags: 0,
    passphrase: passphrase.as_CFTypeRef(),
    alertTitle: alert_title.as_concrete_TypeRef(),
    alertPrompt: std::ptr::null(),
    accessRef: std::ptr::null_mut(),
    keyUsage: std::ptr::null(),
    keyAttributes: std::ptr::null(),
  };

  let mut data = std::ptr::null();
  // SAFETY: all pointers are valid for the duration of the call and the
  // returned data follows the create rule
  let status = unsafe {
    SecItemExport(
      sec_identity.as_CFTypeRef(),
      kSecFormatPKCS12,
      0,
      &params,
      &mut data,
    )
  };
  if status != 0 || data.is_null() {
    return Err(Error::KeychainExport(
      security_framework::base::Error::from_code(status).to_string(),
    ));
  }
  // SAFETY: SecItemExport succeeded so data points to a valid CFData
  let data = unsafe { CFData::wrap_under_create_rule(data) };

  Ok((info, data.bytes().to_vec()))
}

/// Imports a PKCS#12 archive into the login keychain, allowing codesign to use it.
#[cfg(target_os = "macos")]
pub fn import_into_login_keychain(cert_path: &Path, certificate_password: &OsString) -> Result<()> {
  let home_dir = dirs::home_dir().ok_or(Error::ResolveHomeDir)?;
  let keychain_path = home_dir
    .join("Library")
    .join("Keychains")
    .join("login.keychain-db");
  import_certificate(cert_path, certificate_password, &keychain_path)
}

/// Format of the certificate file handed to `security import`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CertificateFormat {
  /// PKCS#12 archive (.p12).
  Pkcs12,
  /// Sequence of PEM items, e.g. a private key followed by its certificate.
  PemSequence,
}

impl CertificateFormat {
  fn detect(contents: &[u8]) -> Self {
    if contents.trim_ascii_start().starts_with(b"-----BEGIN ") {
      Self::PemSequence
    } else {
      Self::Pkcs12
    }
  }

  fn security_format(self) -> &'static str {
    match self {
      Self::Pkcs12 => "pkcs12",
      Self::PemSequence => "pemseq",
    }
  }

  fn description(self) -> &'static str {
    match self {
      Self::Pkcs12 => "a PKCS#12 (.p12) file",
      Self::PemSequence => "a PEM bundle",
    }
  }
}

/// Imports the certificate and its private key into the given keychain.
///
/// The format is passed explicitly because `security` fails to infer it for
/// p12 files using PBES2/AES encryption (the OpenSSL 3 default).
fn import_certificate(
  cert_path: &Path,
  certificate_password: &OsString,
  keychain_path: &Path,
) -> Result<()> {
  let contents = std::fs::read(cert_path).map_err(|error| Error::Fs {
    context: "failed to read certificate file",
    path: cert_path.to_path_buf(),
    error,
  })?;
  let format = CertificateFormat::detect(&contents);

  assert_command(
    Command::new("security")
      .arg("import")
      .arg(cert_path)
      .args(["-f", format.security_format(), "-t", "agg"])
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
      .arg(keychain_path)
      .piped(),
    &format!(
      "failed to import the signing certificate as {}; the certificate must be a PKCS#12 (.p12) file or a PEM bundle containing both the certificate and its private key, and the password must match the one used to export it",
      format.description()
    ),
  )
  .map_err(|error| Error::CommandFailed {
    command: format!("security import -f {}", format.security_format()),
    error,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_certificate_format() {
    assert_eq!(
      CertificateFormat::detect(include_bytes!("../tests/fixtures/pbes2.p12")),
      CertificateFormat::Pkcs12
    );
    assert_eq!(
      CertificateFormat::detect(include_bytes!("../tests/fixtures/bundle.pem")),
      CertificateFormat::PemSequence
    );
    assert_eq!(
      CertificateFormat::detect(b"\n  -----BEGIN CERTIFICATE-----\n"),
      CertificateFormat::PemSequence
    );
  }

  #[cfg(target_os = "macos")]
  #[test]
  fn failed_setup_deletes_keychain() {
    let keychains_dir = dirs::home_dir().unwrap().join("Library").join("Keychains");
    let list_keychains = || {
      std::fs::read_dir(&keychains_dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .filter(|name| name.to_string_lossy().ends_with(".keychain-db"))
        .collect::<std::collections::BTreeSet<_>>()
    };

    let dir = tempfile::tempdir().unwrap();
    let cert_path = dir.path().join("cert.p12");
    std::fs::write(&cert_path, b"not a certificate").unwrap();

    let before = list_keychains();
    assert!(Keychain::with_certificate_file(&cert_path, &"password".into()).is_err());
    let leaked = list_keychains()
      .difference(&before)
      .cloned()
      .collect::<Vec<_>>();

    assert!(leaked.is_empty(), "leaked keychains: {leaked:?}");
  }

  #[cfg(target_os = "macos")]
  mod import {
    use super::super::*;

    const PASSWORD: &str = "tauri";

    /// Temporary keychain that is not added to the user's search list.
    struct TempKeychain {
      _dir: tempfile::TempDir,
      path: PathBuf,
    }

    impl TempKeychain {
      fn new() -> Self {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.keychain-db");
        let status = Command::new("security")
          .args(["create-keychain", "-p", "test"])
          .arg(&path)
          .status()
          .unwrap();
        assert!(status.success(), "failed to create test keychain");
        Self { _dir: dir, path }
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

    fn fixture(name: &str) -> PathBuf {
      Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
    }

    fn import(name: &str, password: &str) -> Result<()> {
      let keychain = TempKeychain::new();
      import_certificate(&fixture(name), &password.into(), &keychain.path)
    }

    // regression test for https://github.com/tauri-apps/tauri/issues/15843
    #[test]
    fn imports_pbes2_p12() {
      import("pbes2.p12", PASSWORD).unwrap();
    }

    #[test]
    fn imports_legacy_p12() {
      import("legacy.p12", PASSWORD).unwrap();
    }

    #[test]
    fn imports_pem_bundle() {
      import("bundle.pem", PASSWORD).unwrap();
    }

    #[test]
    fn imports_generated_p12() {
      let identities =
        crate::certificate::parse_pkcs12(&std::fs::read(fixture("legacy.p12")).unwrap(), PASSWORD)
          .unwrap();
      let p12 = identities[0].to_pkcs12("generated").unwrap();

      let dir = tempfile::tempdir().unwrap();
      let path = dir.path().join("generated.p12");
      std::fs::write(&path, p12).unwrap();

      let keychain = TempKeychain::new();
      import_certificate(&path, &"generated".into(), &keychain.path).unwrap();
    }

    #[test]
    fn wrong_password_error_explains_requirements() {
      let error = import("pbes2.p12", "wrong").unwrap_err().to_string();
      assert!(error.contains("security import -f pkcs12"), "{error}");
      assert!(error.contains("password must match"), "{error}");
    }
  }
}
