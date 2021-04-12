// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod extract;
mod file_move;

use std::{fs, path::Path};

pub use extract::*;
pub use file_move::*;

/// Reads a string file.
pub fn read_string<P: AsRef<Path>>(file: P) -> crate::api::Result<String> {
  fs::read_to_string(file).map_err(Into::into)
}

/// Reads a binary file.
pub fn read_binary<P: AsRef<Path>>(file: P) -> crate::api::Result<Vec<u8>> {
  fs::read(file).map_err(Into::into)
}

#[cfg(test)]
mod test {
  use super::*;
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

    if let Error::Io(e) = res.unwrap_err() {
      #[cfg(windows)]
      assert_eq!(e.to_string(), "Access is denied. (os error 5)".to_string());
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

    if let Error::Io(e) = res.unwrap_err() {
      #[cfg(windows)]
      assert_eq!(e.to_string(), "Access is denied. (os error 5)".to_string());
      #[cfg(not(windows))]
      assert_eq!(e.to_string(), "Is a directory (os error 21)".to_string());
    }
  }
}
