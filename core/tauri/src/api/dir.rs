// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to file system directory management.

use serde::Serialize;
use std::{
  fs::{self, metadata, symlink_metadata},
  path::{Path, PathBuf},
};
use tempfile::{self, tempdir};

/// A disk entry which is either a file, a directory or a symlink.
///
/// This is the result of the [`read_dir`].
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct DirEntry {
  /// The name of the entry (file name with extension or directory name).
  pub name: Option<String>,
  /// Specifies whether this entry is a directory or not.
  pub is_directory: bool,
  /// Specifies whether this entry is a file or not.
  pub is_file: bool,
  /// Specifies whether this entry is a symlink or not.
  pub is_symlink: bool,
}

/// Checks if the given path is a directory.
pub fn is_dir<P: AsRef<Path>>(path: P) -> crate::api::Result<bool> {
  fs::metadata(path).map(|md| md.is_dir()).map_err(Into::into)
}

fn is_symlink<P: AsRef<Path>>(path: P) -> crate::api::Result<bool> {
  // TODO: remove the different implementation once we raise tauri's MSRV to at least 1.58
  #[cfg(windows)]
  let ret = symlink_metadata(path)
    .map(|md| md.is_symlink())
    .map_err(Into::into);

  #[cfg(not(windows))]
  let ret = symlink_metadata(path)
    .map(|md| md.file_type().is_symlink())
    .map_err(Into::into);

  ret
}

/// Reads a directory.
pub fn read_dir<P: AsRef<Path>>(path: P) -> crate::api::Result<Vec<DirEntry>> {
  let mut files_and_dirs: Vec<DirEntry> = vec![];
  for entry in fs::read_dir(path)? {
    let path = entry?.path();
    let file_type = path.metadata()?.file_type();
    files_and_dirs.push(DirEntry {
      is_directory: file_type.is_dir(),
      is_file: file_type.is_file(),
      is_symlink: is_symlink(&path).unwrap_or(false),
      name: path
        .file_name()
        .map(|name| name.to_string_lossy())
        .map(|name| name.to_string()),
    });
  }
  Result::Ok(files_and_dirs)
}

/// Runs a closure with a temporary directory argument.
pub fn with_temp_dir<F: FnOnce(&tempfile::TempDir)>(callback: F) -> crate::api::Result<()> {
  let dir = tempdir()?;
  callback(&dir);
  dir.close()?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use quickcheck_macros::quickcheck;
  use std::path::PathBuf;

  // check is dir function by passing in arbitrary strings
  #[quickcheck]
  fn qc_is_dir(f: String) -> bool {
    // if the string runs through is_dir and comes out as an OK result then it must be a DIR.
    if is_dir(f.clone()).is_ok() {
      PathBuf::from(f).is_dir()
    } else {
      true
    }
  }

  #[test]
  // check the read_dir function
  fn check_read_dir() {
    // define a relative directory test/api/
    let dir = PathBuf::from("test/api/");

    // call list_dir_contents on the dir
    let res = read_dir(dir);

    // assert that the result is Ok()
    assert!(res.is_ok());

    // destruct the vector from the Ok()
    if let Ok(vec) = res {
      // assert the length of the vector is 2
      assert_eq!(vec.len(), 2);

      // get the two DiskEntry structs in this vector
      let first = &vec[0];
      let second = &vec[1];

      if first.name == Some(String::from("test.txt")) {
        // check the fields for the first DiskEntry
        assert!(!first.is_directory);
        assert!(first.is_file);
        assert_eq!(first.name, Some("test.txt".to_string()));

        // check the fields for the second DiskEntry
        assert!(!second.is_directory);
        assert!(second.is_file);
        assert_eq!(second.name, Some("test_binary".to_string()));
      } else {
        // check the fields for the first DiskEntry
        assert!(!second.is_directory);
        assert!(second.is_file);
        assert_eq!(second.name, Some("test.txt".to_string()));

        // check the fields for the second DiskEntry
        assert!(!first.is_directory);
        assert!(first.is_file);
        assert_eq!(first.name, Some("test_binary".to_string()));
      }
    }
  }

  #[test]
  // test the with_temp_dir function
  fn check_test_dir() {
    // create a callback closure that takes in a TempDir type and prints it.
    let callback = |td: &tempfile::TempDir| {
      println!("{td:?}");
    };

    // execute the with_temp_dir function on the callback
    let res = with_temp_dir(callback);

    // assert that the result is an OK type.
    assert!(res.is_ok());
  }
}
