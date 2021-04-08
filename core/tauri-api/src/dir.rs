use serde::Serialize;
use std::{
  fs::{self, metadata},
  path::{Path, PathBuf},
};
use tempfile::{self, tempdir};

/// The result of the `read_dir` function.
///
/// A DiskEntry is either a file or a directory.
/// The `children` Vec is always `Some` if the entry is a directory.
#[derive(Debug, Serialize)]
pub struct DiskEntry {
  /// The path to this entry.
  pub path: PathBuf,
  /// The name of this entry (file name with extension or directory name)
  pub name: Option<String>,
  /// The children of this entry if it's a directory.
  #[serde(skip_serializing_if = "Option::is_none")]
  pub children: Option<Vec<DiskEntry>>,
}

/// Checks if the given path is a directory.
pub fn is_dir<P: AsRef<Path>>(path: P) -> crate::Result<bool> {
  metadata(path).map(|md| md.is_dir()).map_err(Into::into)
}

/// Reads a directory. Can perform recursive operations.
pub fn read_dir<P: AsRef<Path>>(path: P, recursive: bool) -> crate::Result<Vec<DiskEntry>> {
  let mut files_and_dirs: Vec<DiskEntry> = vec![];
  for entry in fs::read_dir(path)? {
    let path = entry?.path();
    let path_as_string = path.display().to_string();

    if let Ok(flag) = is_dir(&path_as_string) {
      files_and_dirs.push(DiskEntry {
        path: path.clone(),
        children: if flag {
          Some(if recursive {
            read_dir(&path_as_string, true)?
          } else {
            vec![]
          })
        } else {
          None
        },
        name: path
          .file_name()
          .map(|name| name.to_string_lossy())
          .map(|name| name.to_string()),
      });
    }
  }
  Result::Ok(files_and_dirs)
}

/// Runs a closure with a temp dir argument.
pub fn with_temp_dir<F: FnOnce(&tempfile::TempDir)>(callback: F) -> crate::Result<()> {
  let dir = tempdir()?;
  callback(&dir);
  dir.close()?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use quickcheck_macros::quickcheck;
  use std::{ffi::OsStr, path::PathBuf};

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

  fn name_from_path(path: PathBuf) -> Option<String> {
    path
      .file_name()
      .map(|name| name.to_string_lossy())
      .map(|name| name.to_string())
  }

  #[test]
  // check the read_dir function with recursive = true
  fn check_read_dir_recursively() {
    // define a relative directory string test/
    let dir = PathBuf::from("test/");
    // add the files to this directory
    let mut file_one = dir.clone();
    file_one.push("test.txt");
    let mut file_two = dir.clone();
    file_two.push("test_binary");

    // call walk_dir on the directory
    let res = read_dir(dir, true);

    // assert that the result is Ok()
    assert!(res.is_ok());

    // destruct the OK into a vector of DiskEntry Structs
    if let Ok(vec) = res {
      // assert that the vector length is only 3
      assert_eq!(vec.len(), 2);

      // get the first DiskEntry
      let first = &vec[0];
      // get the second DiskEntry
      let second = &vec[1];

      if first.path.extension() == Some(OsStr::new("txt")) {
        // check the fields for the first DiskEntry
        assert_eq!(first.path, file_one);
        assert_eq!(first.children.is_some(), false);
        assert_eq!(first.name, name_from_path(file_one));

        // check the fields for the third DiskEntry
        assert_eq!(second.path, file_two);
        assert_eq!(second.children.is_some(), false);
        assert_eq!(second.name, name_from_path(file_two));
      } else {
        // check the fields for the second DiskEntry
        assert_eq!(first.path, file_two);
        assert_eq!(first.children.is_some(), false);
        assert_eq!(first.name, name_from_path(file_two));

        // check the fields for the third DiskEntry
        assert_eq!(second.path, file_one);
        assert_eq!(second.children.is_some(), false);
        assert_eq!(second.name, name_from_path(file_one));
      }
    }
  }

  #[test]
  // check the read_dir function with recursive = false
  fn check_read_dir() {
    // define a relative directory test/
    let dir = PathBuf::from("test/");

    // call list_dir_contents on the dir
    let res = read_dir(dir, false);

    // assert that the result is Ok()
    assert!(res.is_ok());

    // destruct the vector from the Ok()
    if let Ok(vec) = res {
      // assert the length of the vector is 2
      assert_eq!(vec.len(), 2);

      // get the two DiskEntry structs in this vector
      let first = &vec[0];
      let second = &vec[1];

      if first.path.extension() == Some(OsStr::new("txt")) {
        // check the fields for the first DiskEntry
        assert_eq!(first.path, PathBuf::from("test/test.txt"));
        assert_eq!(first.children.is_some(), false);
        assert_eq!(first.name, Some("test.txt".to_string()));

        // check the fields for the second DiskEntry
        assert_eq!(second.path, PathBuf::from("test/test_binary"));
        assert_eq!(second.children.is_some(), false);
        assert_eq!(second.name, Some("test_binary".to_string()));
      } else {
        // check the fields for the first DiskEntry
        assert_eq!(second.path, PathBuf::from("test/test.txt"));
        assert_eq!(second.children.is_some(), false);
        assert_eq!(second.name, Some("test.txt".to_string()));

        // check the fields for the second DiskEntry
        assert_eq!(first.path, PathBuf::from("test/test_binary"));
        assert_eq!(first.children.is_some(), false);
        assert_eq!(first.name, Some("test_binary".to_string()));
      }
    }
  }

  #[test]
  // test the with_temp_dir function
  fn check_test_dir() {
    // create a callback closure that takes in a TempDir type and prints it.
    let callback = |td: &tempfile::TempDir| {
      println!("{:?}", td);
    };

    // execute the with_temp_dir function on the callback
    let res = with_temp_dir(callback);

    // assert that the result is an OK type.
    assert!(res.is_ok());
  }
}
