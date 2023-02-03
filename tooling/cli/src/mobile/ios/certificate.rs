use crate::Result;
use tauri_bundler::bundle::macos_sign::{delete_keychain, setup_keychain};

pub fn setup() -> Result<()> {
  if let (Some(certificate_encoded), Some(certificate_password)) = (
    std::env::var_os("APPLE_CERTIFICATE"),
    std::env::var_os("APPLE_CERTIFICATE_PASSWORD"),
  ) {
    // setup keychain allow you to import your certificate
    // for CI build
    setup_keychain(certificate_encoded, certificate_password)?;
    Ok(())
  } else {
    Err(anyhow::anyhow!(
      "Missing APPLE_CERTIFICATE and APPLE_CERTIFICATE_PASSWORD environment variables"
    ))
  }
}

pub fn delete() {
  delete_keychain();
}
