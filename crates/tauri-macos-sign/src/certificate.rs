// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::apple_codesign::create_self_signed_code_signing_certificate;
use rsa::pkcs8::EncodePrivateKey;
use x509_certificate::{
  CapturedX509Certificate, EcdsaCurve, KeyAlgorithm, Sign, X509CertificateBuilder,
};

/// PKCS#9 emailAddress attribute (1.2.840.113549.1.9.1).
const OID_EMAIL_ADDRESS: &[u8] = &[42, 134, 72, 134, 247, 13, 1, 9, 1];

pub use crate::apple_codesign::CertificateProfile;

use crate::{Error, Result};

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
  )
  .map_err(|error| Error::FailedToCreateSelfSignedCertificate {
    error: Box::new(error),
  })?;

  let pfx = p12::PFX::new(
    &cert
      .encode_der()
      .map_err(|error| Error::FailedToEncodeDER { error })?,
    &key_pair.to_pkcs8_one_asymmetric_key_der(),
    None,
    &request.password,
    "code-signing",
  )
  .ok_or(Error::FailedToCreatePFX)?;
  let der = pfx.to_der();

  Ok(der)
}

/// A certificate and its private key, e.g. read from a PKCS#12 (.p12/.pfx) file.
pub struct Identity {
  pub certificate: CapturedX509Certificate,
  /// PKCS#8 DER encoded private key.
  pub private_key: Vec<u8>,
  pub friendly_name: Option<String>,
}

impl Identity {
  pub fn info(&self) -> Result<CertificateInfo> {
    CertificateInfo::from_certificate(&self.certificate)
  }

  /// Encodes this identity as a PKCS#12 archive protected by the given password.
  pub fn to_pkcs12(&self, password: &str) -> Result<Vec<u8>> {
    let friendly_name = match &self.friendly_name {
      Some(name) => name.clone(),
      None => self
        .certificate
        .subject_common_name()
        .unwrap_or_else(|| "code-signing".into()),
    };
    create_pkcs12(
      &self.certificate,
      &self.private_key,
      password,
      &friendly_name,
    )
  }
}

/// Summary of the fields of a certificate that matter for code signing.
#[derive(Debug, Clone)]
pub struct CertificateInfo {
  pub common_name: Option<String>,
  pub organization: Option<String>,
  /// For Apple certificates this is the Team ID.
  pub organizational_unit: Option<String>,
  /// SHA-1 hash of the DER encoded certificate as uppercase hex,
  /// known as the certificate thumbprint on Windows.
  pub sha1_thumbprint: String,
  pub not_after: chrono::DateTime<chrono::Utc>,
}

impl CertificateInfo {
  pub fn from_certificate(certificate: &CapturedX509Certificate) -> Result<Self> {
    let subject = certificate.subject_name();
    let sha1 = certificate
      .sha1_fingerprint()
      .map_err(|error| Error::FailedToEncodeDER { error })?;
    Ok(Self {
      common_name: certificate.subject_common_name(),
      organization: subject
        .iter_organization()
        .next()
        .and_then(|v| v.to_string().ok()),
      organizational_unit: subject
        .iter_organizational_unit()
        .next()
        .and_then(|v| v.to_string().ok()),
      sha1_thumbprint: hex::encode_upper(sha1.as_ref()),
      not_after: certificate.validity_not_after(),
    })
  }
}

/// Generates a new 2048-bit RSA private key, the key type Apple requires for code signing
/// certificates. Returns the PKCS#8 DER encoded key.
pub fn generate_rsa_private_key() -> Result<Vec<u8>> {
  let key = rsa::RsaPrivateKey::new(&mut rand08::thread_rng(), 2048)
    .map_err(|e| Error::PrivateKey(format!("failed to generate RSA key: {e}")))?;
  key
    .to_pkcs8_der()
    .map(|der| der.as_bytes().to_vec())
    .map_err(|e| Error::PrivateKey(format!("failed to encode RSA key: {e}")))
}

/// Encodes a PKCS#8 DER private key as PEM.
pub fn private_key_to_pem(private_key: &[u8]) -> String {
  pem::encode(&pem::Pem::new("PRIVATE KEY", private_key.to_vec()))
}

/// Parses a private key in PEM or DER format, either PKCS#8 or PKCS#1 (RSA).
/// Returns the PKCS#8 DER encoded key.
pub fn parse_private_key(data: &[u8]) -> Result<Vec<u8>> {
  use rsa::pkcs1::DecodeRsaPrivateKey;

  let (label, der) = if data.trim_ascii_start().starts_with(b"-----BEGIN ") {
    let pem = pem::parse(data).map_err(|e| Error::PrivateKey(format!("invalid PEM: {e}")))?;
    (Some(pem.tag().to_string()), pem.into_contents())
  } else {
    (None, data.to_vec())
  };

  match label.as_deref() {
    Some("ENCRYPTED PRIVATE KEY") => Err(Error::PrivateKey(
      "the private key is encrypted; decrypt it first with `openssl pkcs8 -in key.pem -out decrypted.pem`".into(),
    )),
    Some("PRIVATE KEY") => {
      x509_certificate::InMemorySigningKeyPair::from_pkcs8_der(&der)
        .map_err(|error| Error::X509Certificate { error })?;
      Ok(der)
    }
    Some("RSA PRIVATE KEY") => rsa_pkcs1_to_pkcs8(&der),
    Some(label) => Err(Error::PrivateKey(format!(
      "unsupported private key type `{label}`; convert it to PKCS#8 with `openssl pkcs8 -topk8 -nocrypt -in key.pem -out key-pkcs8.pem`"
    ))),
    None => {
      if x509_certificate::InMemorySigningKeyPair::from_pkcs8_der(&der).is_ok() {
        Ok(der)
      } else if rsa::RsaPrivateKey::from_pkcs1_der(&der).is_ok() {
        rsa_pkcs1_to_pkcs8(&der)
      } else {
        Err(Error::PrivateKey(
          "unrecognized private key format, expected a PKCS#8 or PKCS#1 key".into(),
        ))
      }
    }
  }
}

fn rsa_pkcs1_to_pkcs8(der: &[u8]) -> Result<Vec<u8>> {
  use rsa::pkcs1::DecodeRsaPrivateKey;
  let key = rsa::RsaPrivateKey::from_pkcs1_der(der)
    .map_err(|e| Error::PrivateKey(format!("invalid RSA private key: {e}")))?;
  key
    .to_pkcs8_der()
    .map(|der| der.as_bytes().to_vec())
    .map_err(|e| Error::PrivateKey(format!("failed to encode RSA key: {e}")))
}

/// Parses a X.509 certificate in PEM or DER format (e.g. a `.cer` file downloaded from Apple).
pub fn parse_certificate(data: &[u8]) -> Result<CapturedX509Certificate> {
  if data.trim_ascii_start().starts_with(b"-----BEGIN ") {
    CapturedX509Certificate::from_pem(data)
  } else {
    CapturedX509Certificate::from_der(data.to_vec())
  }
  .map_err(|error| Error::X509Certificate { error })
}

/// Creates a PEM encoded certificate signing request (CSR) for the given private key,
/// to be uploaded to the Apple Developer portal when creating a certificate.
pub fn create_certificate_signing_request(
  private_key: &[u8],
  common_name: &str,
  email: Option<&str>,
) -> Result<String> {
  let key_pair = x509_certificate::InMemorySigningKeyPair::from_pkcs8_der(private_key)
    .map_err(|error| Error::X509Certificate { error })?;

  let mut builder = X509CertificateBuilder::default();
  builder
    .subject()
    .append_common_name_utf8_string(common_name)
    .map_err(|e| Error::PrivateKey(format!("invalid common name: {e:?}")))?;
  if let Some(email) = email {
    builder
      .subject()
      .append_utf8_string(
        bcder::Oid(bytes::Bytes::from_static(OID_EMAIL_ADDRESS)),
        email,
      )
      .map_err(|e| Error::PrivateKey(format!("invalid email address: {e:?}")))?;
  }

  builder
    .create_certificate_signing_request(&key_pair)
    .and_then(|csr| csr.encode_pem().map_err(Into::into))
    .map_err(|error| Error::X509Certificate { error })
}

/// Creates a PKCS#12 archive containing the certificate and its private key.
///
/// Fails if the private key does not belong to the certificate.
pub fn create_pkcs12(
  certificate: &CapturedX509Certificate,
  private_key: &[u8],
  password: &str,
  friendly_name: &str,
) -> Result<Vec<u8>> {
  let key_pair = x509_certificate::InMemorySigningKeyPair::from_pkcs8_der(private_key)
    .map_err(|error| Error::X509Certificate { error })?;
  if key_pair.public_key_data() != certificate.public_key_data() {
    return Err(Error::PrivateKey(
      "the private key does not match the certificate's public key".into(),
    ));
  }

  let pfx = p12::PFX::new(
    &certificate
      .encode_der()
      .map_err(|error| Error::FailedToEncodeDER { error })?,
    private_key,
    None,
    password,
    friendly_name,
  )
  .ok_or(Error::FailedToCreatePFX)?;
  Ok(pfx.to_der())
}

/// Reads every certificate with a matching private key from a PKCS#12 archive.
pub fn parse_pkcs12(data: &[u8], password: &str) -> Result<Vec<Identity>> {
  let pfx = p12::PFX::parse(data)
    .map_err(|e| Error::Pkcs12(format!("the file does not appear to be PKCS#12: {e:?}")))?;
  if !pfx.verify_mac(password) {
    return Err(Error::Pkcs12("incorrect password".into()));
  }
  let bmp_password = bmp_string(password);
  let bags = safe_bags(&pfx, &bmp_password)?;

  let mut certificates = Vec::new();
  let mut keys = Vec::new();
  for bag in &bags {
    if let Some(der) = bag.bag.get_x509_cert() {
      let certificate =
        CapturedX509Certificate::from_der(der).map_err(|error| Error::X509Certificate { error })?;
      certificates.push((bag.local_key_id(), bag.friendly_name(), certificate));
    } else if let p12::SafeBagKind::Pkcs8ShroudedKeyBag(_) = &bag.bag {
      let key = bag
        .bag
        .get_key(&bmp_password)
        .ok_or_else(|| Error::Pkcs12("failed to decrypt the private key".into()))?;
      keys.push((bag.local_key_id(), key));
    }
  }

  if keys.is_empty() {
    return Err(Error::Pkcs12(
      "the file does not contain a private key".into(),
    ));
  }

  // a single key without a local key id belongs to the single (leaf) certificate
  if let [(None, _)] = keys.as_slice() {
    let (_, private_key) = keys.remove(0);
    let (_, friendly_name, certificate) = certificates
      .into_iter()
      .next()
      .ok_or_else(|| Error::Pkcs12("the file does not contain a certificate".into()))?;
    return Ok(vec![Identity {
      certificate,
      private_key,
      friendly_name,
    }]);
  }

  // pair keys and certificates by their local key id, which is how Keychain Access
  // and `security export` match them when exporting several identities at once
  let identities = keys
    .into_iter()
    .filter_map(|(key_id, private_key)| {
      let key_id = key_id?;
      certificates
        .iter()
        .find(|(cert_key_id, _, _)| cert_key_id.as_ref() == Some(&key_id))
        .map(|(_, friendly_name, certificate)| Identity {
          certificate: certificate.clone(),
          private_key,
          friendly_name: friendly_name.clone(),
        })
    })
    .collect::<Vec<_>>();

  if identities.is_empty() {
    return Err(Error::Pkcs12(
      "the file does not contain a certificate matching its private key".into(),
    ));
  }
  Ok(identities)
}

/// Decrypts and reads the bags of a PKCS#12 archive.
///
/// Unlike [`p12::PFX::bags`], this accepts BER encoded contents, which is what the macOS
/// keychain produces when exporting identities.
fn safe_bags(pfx: &p12::PFX, bmp_password: &[u8]) -> Result<Vec<p12::SafeBag>> {
  let p12::ContentInfo::Data(data) = &pfx.auth_safe else {
    return Err(Error::Pkcs12("unexpected PKCS#12 content type".into()));
  };
  let contents = yasna::parse_ber(data, |r| r.collect_sequence_of(p12::ContentInfo::parse))
    .map_err(|e| Error::Pkcs12(format!("failed to parse contents: {e:?}")))?;

  let mut bags = Vec::new();
  for content in contents {
    let data = content.data(bmp_password).ok_or_else(|| {
      Error::Pkcs12(
        "failed to decrypt the file; PBES2 (AES) encrypted files are not supported, re-export it with legacy encryption (e.g. `openssl pkcs12 -legacy`)".into(),
      )
    })?;
    bags.extend(
      yasna::parse_ber(&data, |r| r.collect_sequence_of(p12::SafeBag::parse))
        .map_err(|e| Error::Pkcs12(format!("failed to parse bags: {e:?}")))?,
    );
  }
  Ok(bags)
}

/// Encodes a password as a null terminated BMPString, as required by PKCS#12.
fn bmp_string(s: &str) -> Vec<u8> {
  let mut bytes = s
    .encode_utf16()
    .flat_map(u16::to_be_bytes)
    .collect::<Vec<u8>>();
  bytes.extend_from_slice(&[0, 0]);
  bytes
}

#[cfg(test)]
mod tests {
  use super::*;

  fn self_signed() -> (CapturedX509Certificate, Vec<u8>) {
    let (cert, key_pair) = create_self_signed_code_signing_certificate(
      KeyAlgorithm::Rsa,
      CertificateProfile::AppleDevelopment,
      "TEAMID1234",
      "Tauri Test",
      "US",
      chrono::Duration::days(1),
    )
    .unwrap();
    (cert, key_pair.to_pkcs8_one_asymmetric_key_der().to_vec())
  }

  #[test]
  fn csr_from_generated_key() {
    let key = generate_rsa_private_key().unwrap();
    let pem = create_certificate_signing_request(&key, "Tauri", Some("dev@tauri.app")).unwrap();
    assert!(pem.starts_with("-----BEGIN CERTIFICATE REQUEST-----"));

    // the PEM roundtrip yields the same key
    assert_eq!(
      parse_private_key(private_key_to_pem(&key).as_bytes()).unwrap(),
      key
    );
  }

  #[test]
  fn parses_pkcs1_rsa_key() {
    use rsa::pkcs1::EncodeRsaPrivateKey;
    use rsa::pkcs8::DecodePrivateKey;
    let key = generate_rsa_private_key().unwrap();
    let rsa_key = rsa::RsaPrivateKey::from_pkcs8_der(&key).unwrap();
    let pkcs1_pem = rsa_key.to_pkcs1_pem(Default::default()).unwrap();
    let parsed = parse_private_key(pkcs1_pem.as_bytes()).unwrap();
    assert_eq!(
      rsa::RsaPrivateKey::from_pkcs8_der(&parsed).unwrap(),
      rsa_key
    );
  }

  #[test]
  fn pkcs12_roundtrip() {
    let (cert, key) = self_signed();
    let p12 = create_pkcs12(&cert, &key, "secret", "Tauri Test").unwrap();

    let identities = parse_pkcs12(&p12, "secret").unwrap();
    assert_eq!(identities.len(), 1);
    assert_eq!(identities[0].certificate, cert);
    assert_eq!(identities[0].friendly_name.as_deref(), Some("Tauri Test"));

    let info = identities[0].info().unwrap();
    assert_eq!(info.organizational_unit.as_deref(), Some("TEAMID1234"));
    assert_eq!(info.sha1_thumbprint.len(), 40);

    assert!(parse_pkcs12(&p12, "wrong").is_err());
  }

  #[test]
  fn pkcs12_rejects_mismatched_key() {
    let (cert, _) = self_signed();
    let other_key = generate_rsa_private_key().unwrap();
    assert!(create_pkcs12(&cert, &other_key, "secret", "Tauri Test").is_err());
  }

  #[test]
  fn parses_legacy_fixture() {
    let identities = parse_pkcs12(include_bytes!("../tests/fixtures/legacy.p12"), "tauri").unwrap();
    assert_eq!(identities.len(), 1);
  }

  #[test]
  fn parses_certificate_pem_and_der() {
    let (cert, _) = self_signed();
    let der = cert.encode_der().unwrap();
    assert_eq!(parse_certificate(&der).unwrap(), cert);
    assert_eq!(
      parse_certificate(cert.encode_pem().as_bytes()).unwrap(),
      cert
    );
  }
}
