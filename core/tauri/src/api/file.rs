// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to file operations.

use std::{fs, path::Path};

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
  #[cfg(not(windows))]
  use crate::api::Error;

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

    let expected_vec = vec![
      71, 73, 70, 56, 57, 97, 1, 0, 1, 0, 128, 0, 0, 255, 255, 255, 0, 0, 0, 33, 249, 4, 1, 0, 0,
      0, 0, 44, 0, 0, 0, 0, 1, 0, 1, 0, 0, 2, 2, 68, 1, 0, 59,
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
