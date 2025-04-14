// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::{Context, Result};
use apple_codesign::create_self_signed_code_signing_certificate;
use x509_certificate::{EcdsaCurve, KeyAlgorithm};

pub use apple_codesign::CertificateProfile;

/// Self signed certificate options.
pub struct SelfSignedCertificateRequest {
  /// Which key type to use
  pub algorithm: String,

  /// Profile
  pub profile: CertificateProfile,

  /// Team ID (this is a short string attached to your Apple Developer account)
  pub team_id: String,

  /// The name of the person this certificate is for
  pub person_name: String,

  /// Country Name (C) value for certificate identifier
  pub country_name: String,

  /// How many days the certificate should be valid for
  pub validity_days: i64,

  /// Certificate password.
  pub password: String,
}

pub fn generate_self_signed(request: SelfSignedCertificateRequest) -> Result<Vec<u8>> {
  let algorithm = match request.algorithm.as_str() {
    "ecdsa" => KeyAlgorithm::Ecdsa(EcdsaCurve::Secp256r1),
    "ed25519" => KeyAlgorithm::Ed25519,
    "rsa" => KeyAlgorithm::Rsa,
    value => panic!("algorithm values should have been validated by arg parser: {value}"),
  };

  let validity_duration = chrono::Duration::days(request.validity_days);

  let (cert, key_pair) = create_self_signed_code_signing_certificate(
    algorithm,
    request.profile,
    &request.team_id,
    &request.person_name,
    &request.country_name,
    validity_duration,
  )?;

  let pfx = p12::PFX::new(
    &cert.encode_der()?,
    &key_pair.to_pkcs8_one_asymmetric_key_der(),
    None,
    &request.password,
    "code-signing",
  )
  .context("failed to create PFX structure")?;
  let der = pfx.to_der();

  Ok(der)
}
