use serde::{Deserialize, Deserializer};
use std::num::NonZeroU8;
use thiserror::Error;

const IDENTIFIER_SEPARATOR: u8 = b':';
const PLUGIN_PREFIX: &str = "tauri-plugin-";

// https://doc.rust-lang.org/cargo/reference/manifest.html#the-name-field
const MAX_LEN_PREFIX: usize = 64 - PLUGIN_PREFIX.len();
const MAX_LEN_BASE: usize = 64;
const MAX_LEN_IDENTIFIER: usize = MAX_LEN_PREFIX + 1 + MAX_LEN_BASE;

#[derive(Debug)]
pub struct Identifier {
  inner: String,
  separator: Option<NonZeroU8>,
}

impl AsRef<str> for Identifier {
  #[inline(always)]
  fn as_ref(&self) -> &str {
    &self.inner
  }
}

impl Identifier {
  #[inline(always)]
  pub fn get(&self) -> &str {
    self.as_ref()
  }

  pub fn get_base(&self) -> &str {
    match self.separator_index() {
      None => self.get(),
      Some(i) => &self.inner[i + 1..],
    }
  }

  pub fn get_prefix(&self) -> Option<&str> {
    self.separator_index().map(|i| &self.inner[0..i])
  }

  pub fn set_prefix(&mut self) -> Result<(), ParseIdentifierError> {
    todo!()
  }

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
  fn lower_alpha(byte: u8) -> Option<Self> {
    byte.is_ascii_lowercase().then_some(Self::Byte(byte))
  }

  fn lower_alpha_hyphen(byte: u8) -> Option<Self> {
    matches!(byte, b'a'..=b'z' | b'-').then_some(Self::Byte(byte))
  }

  fn next(&self, next: u8) -> Option<ValidByte> {
    match (self, next) {
      (ValidByte::Byte(b'-'), IDENTIFIER_SEPARATOR) => None,
      (ValidByte::Separator, b'-') => None,

      (_, IDENTIFIER_SEPARATOR) => Some(ValidByte::Separator),
      (ValidByte::Separator, next) => ValidByte::lower_alpha(next),
      (ValidByte::Byte(b'-'), next) => ValidByte::lower_alpha(next),
      (ValidByte::Byte(_), next) => ValidByte::lower_alpha_hyphen(next),
    }
  }
}

#[derive(Debug, Error)]
pub enum ParseIdentifierError {
  #[error("identifiers cannot start with {}", PLUGIN_PREFIX)]
  StartsWithTauriPlugin,

  #[error("identifiers cannot be empty")]
  Empty,

  #[error("identifiers cannot be longer than {}, found {0}", MAX_LEN_IDENTIFIER)]
  Humungous(usize),

  #[error("identifiers can only include lowercase ASCII, hyphens which are not leading or trailing, and a single colon if using a prefix")]
  InvalidFormat,

  #[error(
    "identifiers can only include a single separator '{}'",
    IDENTIFIER_SEPARATOR
  )]
  MultipleSeparators,

  #[error("identifiers cannot have a trailing hyphen")]
  TrailingHyphen,

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

    let mut bytes = value.bytes();
    if bytes.len() > MAX_LEN_IDENTIFIER {
      return Err(Self::Error::Humungous(bytes.len()));
    }

    // grab the first byte only before parsing the rest
    let mut prev = bytes
      .next()
      .and_then(ValidByte::lower_alpha)
      .ok_or(Self::Error::InvalidFormat)?;

    let mut idx = 0;
    let mut seperator = None;
    for byte in bytes {
      idx += 1; // we already consumed first item
      match prev.next(byte) {
        None => return Err(Self::Error::InvalidFormat),
        Some(next @ ValidByte::Byte(_)) => prev = next,
        Some(ValidByte::Separator) => {
          if seperator.is_none() {
            // safe to unwrap because idx starts at 1 and cannot go over MAX_IDENTIFIER_LEN
            seperator = Some(idx.try_into().unwrap());
            prev = ValidByte::Separator
          } else {
            return Err(Self::Error::MultipleSeparators);
          }
        }
      }
    }

    match prev {
      // empty base
      ValidByte::Separator => return Err(Self::Error::TrailingHyphen),

      // trailing hyphen
      ValidByte::Byte(b'-') => return Err(Self::Error::PrefixWithoutBase),

      _ => (),
    }

    Ok(Self {
      inner: value,
      separator: seperator,
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
