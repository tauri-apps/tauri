// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Identifier for plugins.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::num::NonZeroU8;
use thiserror::Error;

const IDENTIFIER_SEPARATOR: u8 = b':';
const PLUGIN_PREFIX: &str = "tauri-plugin-";
const CORE_PLUGIN_IDENTIFIER_PREFIX: &str = "core:";

// <https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field>
const MAX_LEN_PREFIX: usize = 64 - PLUGIN_PREFIX.len();
const MAX_LEN_BASE: usize = 64;
const MAX_LEN_IDENTIFIER: usize = MAX_LEN_PREFIX + 1 + MAX_LEN_BASE;

/// Plugin identifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identifier {
  inner: String,
  separator: Option<NonZeroU8>,
}

#[cfg(feature = "schema")]
impl schemars::JsonSchema for Identifier {
  fn schema_name() -> String {
    "Identifier".to_string()
  }

  fn schema_id() -> std::borrow::Cow<'static, str> {
    // Include the module, in case a type with the same name is in another module/crate
    std::borrow::Cow::Borrowed(concat!(module_path!(), "::Identifier"))
  }

  fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
    String::json_schema(gen)
  }
}

impl AsRef<str> for Identifier {
  #[inline(always)]
  fn as_ref(&self) -> &str {
    &self.inner
  }
}

impl Identifier {
  /// Get the identifier str.
  #[inline(always)]
  pub fn get(&self) -> &str {
    self.as_ref()
  }

  /// Get the identifier without prefix.
  pub fn get_base(&self) -> &str {
    match self.separator_index() {
      None => self.get(),
      Some(i) => &self.inner[i + 1..],
    }
  }

  /// Get the prefix of the identifier.
  pub fn get_prefix(&self) -> Option<&str> {
    self.separator_index().map(|i| &self.inner[0..i])
  }

  /// Set the identifier prefix.
  pub fn set_prefix(&mut self) -> Result<(), ParseIdentifierError> {
    todo!()
  }

  /// Get the identifier string and its separator.
  pub fn into_inner(self) -> (String, Option<NonZeroU8>) {
    (self.inner, self.separator)
  }

  fn separator_index(&self) -> Option<usize> {
    self.separator.map(|i| i.get() as usize)
  }
}

#[derive(Debug)]
enum ValidByte {
  Separator,
  Byte(u8),
}

impl ValidByte {
  fn alpha_numeric(byte: u8) -> Option<Self> {
    byte.is_ascii_alphanumeric().then_some(Self::Byte(byte))
  }

  fn alpha_numeric_hyphen(byte: u8) -> Option<Self> {
    (byte.is_ascii_alphanumeric() || byte == b'-').then_some(Self::Byte(byte))
  }

  fn next(&self, next: u8) -> Option<ValidByte> {
    match (self, next) {
      (ValidByte::Byte(b'-'), IDENTIFIER_SEPARATOR) => None,
      (ValidByte::Separator, b'-') => None,

      (_, IDENTIFIER_SEPARATOR) => Some(ValidByte::Separator),
      (ValidByte::Separator, next) => ValidByte::alpha_numeric(next),
      (ValidByte::Byte(b'-'), next) => ValidByte::alpha_numeric(next),
      (ValidByte::Byte(_), next) => ValidByte::alpha_numeric_hyphen(next),
    }
  }
}

/// Errors that can happen when parsing an identifier.
#[derive(Debug, Error)]
pub enum ParseIdentifierError {
  /// Identifier start with the plugin prefix.
  #[error("identifiers cannot start with {}", PLUGIN_PREFIX)]
  StartsWithTauriPlugin,

  /// Identifier empty.
  #[error("identifiers cannot be empty")]
  Empty,

  /// Identifier is too long.
  #[error("identifiers cannot be longer than {len}, found {0}", len = MAX_LEN_IDENTIFIER)]
  Humongous(usize),

  /// Identifier is not in a valid format.
  #[error("identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix")]
  InvalidFormat,

  /// Identifier has multiple separators.
  #[error(
    "identifiers can only include a single separator '{}'",
    IDENTIFIER_SEPARATOR
  )]
  MultipleSeparators,

  /// Identifier has a trailing hyphen.
  #[error("identifiers cannot have a trailing hyphen")]
  TrailingHyphen,

  /// Identifier has a prefix without a base.
  #[error("identifiers cannot have a prefix without a base")]
  PrefixWithoutBase,
}

impl TryFrom<String> for Identifier {
  type Error = ParseIdentifierError;

  fn try_from(value: String) -> Result<Self, Self::Error> {
    if value.starts_with(PLUGIN_PREFIX) {
      return Err(Self::Error::StartsWithTauriPlugin);
    }

    if value.is_empty() {
      return Err(Self::Error::Empty);
    }

    if value.len() > MAX_LEN_IDENTIFIER {
      return Err(Self::Error::Humongous(value.len()));
    }

    let is_core_identifier = value.starts_with(CORE_PLUGIN_IDENTIFIER_PREFIX);

    let mut bytes = value.bytes();

    // grab the first byte only before parsing the rest
    let mut prev = bytes
      .next()
      .and_then(ValidByte::alpha_numeric)
      .ok_or(Self::Error::InvalidFormat)?;

    let mut idx = 0;
    let mut separator = None;
    for byte in bytes {
      idx += 1; // we already consumed first item
      match prev.next(byte) {
        None => return Err(Self::Error::InvalidFormat),
        Some(next @ ValidByte::Byte(_)) => prev = next,
        Some(ValidByte::Separator) => {
          if separator.is_none() || is_core_identifier {
            // safe to unwrap because idx starts at 1 and cannot go over MAX_IDENTIFIER_LEN
            separator = Some(idx.try_into().unwrap());
            prev = ValidByte::Separator
          } else {
            return Err(Self::Error::MultipleSeparators);
          }
        }
      }
    }

    match prev {
      // empty base
      ValidByte::Separator => return Err(Self::Error::PrefixWithoutBase),

      // trailing hyphen
      ValidByte::Byte(b'-') => return Err(Self::Error::TrailingHyphen),

      _ => (),
    }

    Ok(Self {
      inner: value,
      separator,
    })
  }
}

impl<'de> Deserialize<'de> for Identifier {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    Self::try_from(String::deserialize(deserializer)?).map_err(serde::de::Error::custom)
  }
}

impl Serialize for Identifier {
  fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(self.get())
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  fn ident(s: impl Into<String>) -> Result<Identifier, ParseIdentifierError> {
    Identifier::try_from(s.into())
  }

  #[test]
  fn max_len_fits_in_u8() {
    assert!(MAX_LEN_IDENTIFIER < u8::MAX as usize)
  }

  #[test]
  fn format() {
    assert!(ident("prefix:base").is_ok());
    assert!(ident("prefix3:base").is_ok());
    assert!(ident("preFix:base").is_ok());

    // bad
    assert!(ident("tauri-plugin-prefix:base").is_err());

    assert!(ident("-prefix-:-base-").is_err());
    assert!(ident("-prefix:base").is_err());
    assert!(ident("prefix-:base").is_err());
    assert!(ident("prefix:-base").is_err());
    assert!(ident("prefix:base-").is_err());

    assert!(ident("pre--fix:base--sep").is_err());
    assert!(ident("prefix:base--sep").is_err());
    assert!(ident("pre--fix:base").is_err());

    assert!(ident("prefix::base").is_err());
    assert!(ident(":base").is_err());
    assert!(ident("prefix:").is_err());
    assert!(ident(":prefix:base:").is_err());
    assert!(ident("base:").is_err());

    assert!(ident("").is_err());
    assert!(ident("💩").is_err());

    assert!(ident("a".repeat(MAX_LEN_IDENTIFIER + 1)).is_err());
  }

  #[test]
  fn base() {
    assert_eq!(ident("prefix:base").unwrap().get_base(), "base");
    assert_eq!(ident("base").unwrap().get_base(), "base");
  }

  #[test]
  fn prefix() {
    assert_eq!(ident("prefix:base").unwrap().get_prefix(), Some("prefix"));
    assert_eq!(ident("base").unwrap().get_prefix(), None);
  }
}

#[cfg(feature = "build")]
mod build {
  use proc_macro2::TokenStream;
  use quote::{quote, ToTokens, TokenStreamExt};

  use super::*;

  impl ToTokens for Identifier {
    fn to_tokens(&self, tokens: &mut TokenStream) {
      let s = self.get();
      tokens
        .append_all(quote! { ::tauri::utils::acl::Identifier::try_from(#s.to_string()).unwrap() })
    }
  }
}
