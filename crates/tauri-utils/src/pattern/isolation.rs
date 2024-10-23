// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::array::TryFromSliceError;
use std::borrow::Cow;
use std::fmt::{Debug, Formatter};
use std::string::FromUtf8Error;

use aes_gcm::aead::Aead;
use aes_gcm::{Aes256Gcm, KeyInit, Nonce};
use getrandom::{getrandom, Error as CsprngError};
use serialize_to_javascript::{default_template, Template};

/// The style for the isolation iframe.
pub const IFRAME_STYLE: &str = "#__tauri_isolation__ { display: none !important }";

/// Errors that can occur during Isolation keys generation.
#[derive(Debug, thiserror::Error)]
#[non_exhaustive]
pub enum Error {
  /// Something went wrong with the CSPRNG.
  #[error("CSPRNG error")]
  Csprng(#[from] CsprngError),

  /// Something went wrong with decrypting an AES-GCM payload
  #[error("AES-GCM")]
  Aes,

  /// Nonce was not 96 bits
  #[error("Nonce: {0}")]
  NonceSize(#[from] TryFromSliceError),

  /// Payload was not valid utf8
  #[error("{0}")]
  Utf8(#[from] FromUtf8Error),

  /// Invalid json format
  #[error("{0}")]
  Json(#[from] serde_json::Error),
}

/// A formatted AES-GCM cipher instance along with the key used to initialize it.
#[derive(Clone)]
pub struct AesGcmPair {
  raw: [u8; 32],
  key: Aes256Gcm,
}

impl Debug for AesGcmPair {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    write!(f, "AesGcmPair(...)")
  }
}

impl AesGcmPair {
  fn new() -> Result<Self, Error> {
    let mut raw = [0u8; 32];
    getrandom(&mut raw)?;
    let key = aes_gcm::Key::<Aes256Gcm>::from_slice(&raw);
    Ok(Self {
      raw,
      key: Aes256Gcm::new(key),
    })
  }

  /// The raw value used to create the AES-GCM key
  pub fn raw(&self) -> &[u8; 32] {
    &self.raw
  }

  /// The formatted AES-GCM key
  pub fn key(&self) -> &Aes256Gcm {
    &self.key
  }

  #[doc(hidden)]
  pub fn encrypt(&self, nonce: &[u8; 12], payload: &[u8]) -> Result<Vec<u8>, Error> {
    self
      .key
      .encrypt(nonce.into(), payload)
      .map_err(|_| self::Error::Aes)
  }
}

/// All cryptographic keys required for Isolation encryption
#[derive(Debug, Clone)]
pub struct Keys {
  /// AES-GCM key
  aes_gcm: AesGcmPair,
}

impl Keys {
  /// Securely generate required keys for Isolation encryption.
  pub fn new() -> Result<Self, Error> {
    AesGcmPair::new()
      .map(|aes_gcm| Self { aes_gcm })
      .map_err(Into::into)
  }

  /// The AES-GCM data (and raw data).
  pub fn aes_gcm(&self) -> &AesGcmPair {
    &self.aes_gcm
  }

  /// Decrypts a message using the generated keys.
  pub fn decrypt(&self, raw: RawIsolationPayload<'_>) -> Result<Vec<u8>, Error> {
    let RawIsolationPayload { nonce, payload, .. } = raw;
    let nonce: [u8; 12] = nonce.as_ref().try_into()?;
    self
      .aes_gcm
      .key
      .decrypt(Nonce::from_slice(&nonce), payload.as_ref())
      .map_err(|_| self::Error::Aes)
  }
}

/// Raw representation of
#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawIsolationPayload<'a> {
  nonce: Cow<'a, [u8]>,
  payload: Cow<'a, [u8]>,
  content_type: Cow<'a, str>,
}

impl<'a> RawIsolationPayload<'a> {
  /// Content type of this payload.
  pub fn content_type(&self) -> &Cow<'a, str> {
    &self.content_type
  }
}

impl<'a> TryFrom<&'a Vec<u8>> for RawIsolationPayload<'a> {
  type Error = Error;

  fn try_from(value: &'a Vec<u8>) -> Result<Self, Self::Error> {
    serde_json::from_slice(value).map_err(Into::into)
  }
}

/// The Isolation JavaScript template meant to be injected during codegen.
///
/// Note: This struct is not considered part of the stable API
#[derive(Template)]
#[default_template("isolation.js")]
pub struct IsolationJavascriptCodegen {
  // this template intentionally does not include the runtime field
}

/// The Isolation JavaScript template meant to be injected during runtime.
///
/// Note: This struct is not considered part of the stable API
#[derive(Template)]
#[default_template("isolation.js")]
pub struct IsolationJavascriptRuntime<'a> {
  /// The key used on the Rust backend and the Isolation Javascript
  pub runtime_aes_gcm_key: &'a [u8; 32],
  /// The origin the isolation application is expecting messages from.
  pub origin: String,
  /// The function that processes the IPC message.
  #[raw]
  pub process_ipc_message_fn: &'a str,
}

#[cfg(test)]
mod test {
  #[test]
  fn create_keys() -> Result<(), Box<dyn std::error::Error>> {
    let _ = super::Keys::new()?;
    Ok(())
  }
}
