// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Minimal App Store Connect API client to manage signing certificates,
//! bundle identifiers and provisioning profiles.
//!
//! See <https://developer.apple.com/documentation/appstoreconnectapi>.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::{Result, error::Context};

const API_URL: &str = "https://api.appstoreconnect.apple.com/v1";

/// App Store Connect API team key.
pub struct ApiKey {
  pub key_id: String,
  pub issuer_id: String,
  /// PEM encoded `.p8` private key.
  pub private_key: String,
}

pub struct Client {
  key: ApiKey,
  agent: ureq::Agent,
}

#[derive(Deserialize)]
struct ErrorResponse {
  errors: Vec<ApiError>,
}

#[derive(Deserialize)]
struct ApiError {
  title: Option<String>,
  detail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Resource<A> {
  pub id: String,
  pub attributes: A,
}

#[derive(Deserialize)]
struct Document<D> {
  data: D,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CertificateAttributes {
  /// Base64 encoded DER certificate.
  pub certificate_content: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BundleIdAttributes {
  pub identifier: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileAttributes {
  /// Base64 encoded provisioning profile.
  pub profile_content: Option<String>,
}

impl Client {
  pub fn new(key: ApiKey) -> Result<Self> {
    // validate the key early
    signing_key(&key.private_key)?;
    Ok(Self {
      key,
      agent: crate::helpers::http::agent(false),
    })
  }

  fn token(&self) -> Result<String> {
    use p256::ecdsa::{Signature, signature::Signer};

    let now = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .context("system time is before the UNIX epoch")?
      .as_secs();

    let header = json!({ "alg": "ES256", "kid": self.key.key_id, "typ": "JWT" });
    // tokens are valid for at most 20 minutes
    let claims = json!({
      "iss": self.key.issuer_id,
      "iat": now,
      "exp": now + 15 * 60,
      "aud": "appstoreconnect-v1",
    });

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let message = format!(
      "{}.{}",
      engine.encode(header.to_string()),
      engine.encode(claims.to_string())
    );
    let signature: Signature = signing_key(&self.key.private_key)?.sign(message.as_bytes());
    Ok(format!("{message}.{}", engine.encode(signature.to_bytes())))
  }

  fn request<T: serde::de::DeserializeOwned>(
    &self,
    method: &str,
    path: &str,
    body: Option<Value>,
  ) -> Result<T> {
    let url = format!("{API_URL}{path}");
    let authorization = format!("Bearer {}", self.token()?);

    let response = match body {
      Some(body) => self
        .agent
        .post(&url)
        .header("Authorization", &authorization)
        .header("Content-Type", "application/json")
        .send(body.to_string()),
      None => self
        .agent
        .get(&url)
        .header("Authorization", &authorization)
        .call(),
    }
    .with_context(|| format!("failed to send App Store Connect request {method} {path}"))?;

    let status = response.status();
    let body = response
      .into_body()
      .read_to_string()
      .context("failed to read App Store Connect response")?;

    if !status.is_success() {
      let message = serde_json::from_str::<ErrorResponse>(&body)
        .ok()
        .map(|response| {
          response
            .errors
            .into_iter()
            .map(|e| match (e.title, e.detail) {
              (Some(title), Some(detail)) => format!("{title}: {detail}"),
              (title, detail) => title.or(detail).unwrap_or_default(),
            })
            .collect::<Vec<_>>()
            .join("; ")
        })
        .filter(|m| !m.is_empty())
        .unwrap_or(body);
      crate::error::bail!("App Store Connect request {method} {path} failed ({status}): {message}");
    }

    serde_json::from_str(&body).context("failed to parse App Store Connect response")
  }

  /// Creates a certificate from a PEM encoded certificate signing request.
  pub fn create_certificate(
    &self,
    certificate_type: &str,
    csr_pem: &str,
  ) -> Result<Resource<CertificateAttributes>> {
    self
      .request::<Document<_>>(
        "POST",
        "/certificates",
        Some(json!({
          "data": {
            "type": "certificates",
            "attributes": {
              "certificateType": certificate_type,
              "csrContent": csr_pem,
            }
          }
        })),
      )
      .map(|d| d.data)
  }

  pub fn find_bundle_id(&self, identifier: &str) -> Result<Option<Resource<BundleIdAttributes>>> {
    let path = format!(
      "/bundleIds?filter[identifier]={}&limit=200",
      url::form_urlencoded::byte_serialize(identifier.as_bytes()).collect::<String>()
    );
    // the identifier filter is not an exact match
    let bundle_ids = self
      .request::<Document<Vec<Resource<BundleIdAttributes>>>>("GET", &path, None)?
      .data;
    Ok(
      bundle_ids
        .into_iter()
        .find(|b| b.attributes.identifier == identifier),
    )
  }

  pub fn create_bundle_id(
    &self,
    identifier: &str,
    name: &str,
    platform: &str,
  ) -> Result<Resource<BundleIdAttributes>> {
    self
      .request::<Document<_>>(
        "POST",
        "/bundleIds",
        Some(json!({
          "data": {
            "type": "bundleIds",
            "attributes": {
              "identifier": identifier,
              "name": name,
              "platform": platform,
            }
          }
        })),
      )
      .map(|d| d.data)
  }

  pub fn list_devices(&self, platform: &str) -> Result<Vec<Resource<serde::de::IgnoredAny>>> {
    self
      .request::<Document<Vec<_>>>(
        "GET",
        &format!("/devices?filter[platform]={platform}&filter[status]=ENABLED&limit=200"),
        None,
      )
      .map(|d| d.data)
  }

  pub fn create_profile(
    &self,
    name: &str,
    profile_type: &str,
    bundle_id: &str,
    certificate_id: &str,
    device_ids: &[String],
  ) -> Result<Resource<ProfileAttributes>> {
    let mut relationships = json!({
      "bundleId": { "data": { "type": "bundleIds", "id": bundle_id } },
      "certificates": { "data": [{ "type": "certificates", "id": certificate_id }] },
    });
    if !device_ids.is_empty() {
      relationships["devices"] = json!({
        "data": device_ids
          .iter()
          .map(|id| json!({ "type": "devices", "id": id }))
          .collect::<Vec<_>>()
      });
    }

    self
      .request::<Document<_>>(
        "POST",
        "/profiles",
        Some(json!({
          "data": {
            "type": "profiles",
            "attributes": {
              "name": name,
              "profileType": profile_type,
            },
            "relationships": relationships,
          }
        })),
      )
      .map(|d| d.data)
  }
}

fn signing_key(pem: &str) -> Result<p256::ecdsa::SigningKey> {
  use p256::pkcs8::DecodePrivateKey;
  p256::ecdsa::SigningKey::from_pkcs8_pem(pem.trim()).context(
    "invalid App Store Connect API key, expected the contents of the AuthKey_<KEY_ID>.p8 file",
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn signs_verifiable_token() {
    use p256::{
      ecdsa::{Signature, VerifyingKey, signature::Verifier},
      pkcs8::{EncodePrivateKey, LineEnding},
    };

    let secret = p256::SecretKey::random(&mut p256::elliptic_curve::rand_core::OsRng);
    let pem = secret.to_pkcs8_pem(LineEnding::LF).unwrap();
    let client = Client::new(ApiKey {
      key_id: "KEYID".into(),
      issuer_id: "ISSUER".into(),
      private_key: pem.to_string(),
    })
    .unwrap();

    let token = client.token().unwrap();
    let (message, signature) = token.rsplit_once('.').unwrap();

    let engine = base64::engine::general_purpose::URL_SAFE_NO_PAD;
    let header: Value =
      serde_json::from_slice(&engine.decode(message.split('.').next().unwrap()).unwrap()).unwrap();
    assert_eq!(header["alg"], "ES256");
    assert_eq!(header["kid"], "KEYID");

    let signature = Signature::from_slice(&engine.decode(signature).unwrap()).unwrap();
    VerifyingKey::from(secret.public_key())
      .verify(message.as_bytes(), &signature)
      .unwrap();
  }

  #[test]
  fn rejects_invalid_key() {
    assert!(
      Client::new(ApiKey {
        key_id: "KEYID".into(),
        issuer_id: "ISSUER".into(),
        private_key: "not a key".into(),
      })
      .is_err()
    );
  }
}
