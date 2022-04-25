// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to file operations.

#[cfg(feature = "fs-extract-api")]
mod extract;
mod file_move;

use std::{
  fs,
  path::{Display, Path},
};

#[cfg(feature = "fs-extract-api")]
pub use extract::*;
pub use file_move::*;

use serde::{de::Error as DeError, Deserialize, Deserializer};

#[derive(Clone, Debug)]
pub(crate) struct SafePathBuf(std::path::PathBuf);

impl SafePathBuf {
  pub fn new(path: std::path::PathBuf) -> Result<Self, &'static str> {
    if path
      .components()
      .any(|x| matches!(x, std::path::Component::ParentDir))
    {
      Err("cannot traverse directory, rewrite the path without the use of `../`")
    } else {
      Ok(Self(path))
    }
  }

  #[allow(dead_code)]
  pub unsafe fn new_unchecked(path: std::path::PathBuf) -> Self {
    Self(path)
  }

  #[allow(dead_code)]
  pub fn display(&self) -> Display<'_> {
    self.0.display()
  }
}

impl AsRef<Path> for SafePathBuf {
  fn as_ref(&self) -> &Path {
    self.0.as_ref()
  }
}

impl<'de> Deserialize<'de> for SafePathBuf {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let path = std::path::PathBuf::deserialize(deserializer)?;
    SafePathBuf::new(path).map_err(DeError::custom)
  }
}

/// Reads the entire contents of a file into a string.
pub fn read_string<P: AsRef<Path>>(file: P) -> crate::api::Result<String> {
  fs::read_to_string(file).map_err(Into::into)
}

/// Reads the entire contents of a file into a bytes vector.
pub fn read_binary<P: AsRef<Path>>(file: P) -> crate::api::Result<Vec<u8>> {
  fs::read(file).map_err(Into::into)
}

#[cfg(test)]
mod test {
  use super::*;
  use crate::api::Error;
  use quickcheck::{Arbitrary, Gen};

  use std::path::PathBuf;

  impl Arbitrary for super::SafePathBuf {
    fn arbitrary(g: &mut Gen) -> Self {
      Self(PathBuf::arbitrary(g))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
      Box::new(self.0.shrink().map(SafePathBuf))
    }
  }

  #[test]
  fn check_read_string() {
    let file = String::from("test/api/test.txt");

    let res = read_string(file);

    assert!(res.is_ok());

    if let Ok(s) = res {
      assert_eq!(s, "This is a test doc!".to_string());
    }
  }

  #[test]
  fn check_read_string_fail() {
    let file = String::from("test/api/");

    let res = read_string(file);

    assert!(res.is_err());

    #[cfg(not(windows))]
    if let Error::Io(e) = res.unwrap_err() {
      #[cfg(not(windows))]
      assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
    }
  }

  #[test]
  fn check_read_binary() {
    let file = String::from("test/api/test_binary");

    #[cfg(windows)]
    let expected_vec = vec![
      35, 33, 47, 98, 105, 110, 47, 98, 97, 115, 104, 13, 10, 13, 10, 101, 99, 104, 111, 32, 34,
      72, 101, 108, 108, 111, 32, 116, 104, 101, 114, 101, 34,
    ];
    #[cfg(not(windows))]
    let expected_vec = vec![
      35, 33, 47, 98, 105, 110, 47, 98, 97, 115, 104, 10, 10, 101, 99, 104, 111, 32, 34, 72, 101,
      108, 108, 111, 32, 116, 104, 101, 114, 101, 34,
    ];

    let res = read_binary(file);

    assert!(res.is_ok());

    if let Ok(vec) = res {
      assert_eq!(vec, expected_vec);
    }
  }

  #[test]
  fn check_read_binary_fail() {
    let file = String::from("test/api/");

    let res = read_binary(file);

    assert!(res.is_err());

    #[cfg(not(windows))]
    if let Error::Io(e) = res.unwrap_err() {
      #[cfg(not(windows))]
      assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
    }
  }
}
