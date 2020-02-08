mod utils;

use ignore::Walk;
use serde::Serialize;
use tempfile::{self, tempdir};

use utils::get_dir_name_from_path;

use std::fs::{self, metadata};

#[derive(Debug, Serialize)]
pub struct DiskEntry {
  pub path: String,
  pub is_dir: bool,
  pub name: String,
}

fn is_dir(file_name: String) -> crate::Result<bool> {
  match metadata(file_name) {
    Ok(md) => Result::Ok(md.is_dir()),
    Err(err) => Result::Err(err.to_string().into()),
  }
}

pub fn walk_dir(path_copy: String) -> crate::Result<Vec<DiskEntry>> {
  println!("Trying to walk: {}", path_copy.as_str());
  let mut files_and_dirs: Vec<DiskEntry> = vec![];
  for result in Walk::new(path_copy) {
    if let Ok(entry) = result {
      let display_value = entry.path().display();
      let _dir_name = display_value.to_string();

      if let Ok(flag) = is_dir(display_value.to_string()) {
        files_and_dirs.push(DiskEntry {
          path: display_value.to_string(),
          is_dir: flag,
          name: display_value.to_string(),
        });
      }
    }
  }
  Result::Ok(files_and_dirs)
}

pub fn list_dir_contents(dir_path: String) -> crate::Result<Vec<DiskEntry>> {
  fs::read_dir(dir_path)
    .map_err(|err| crate::Error::with_chain(err, "read string failed"))
    .and_then(|paths| {
      let mut dirs: Vec<DiskEntry> = vec![];
      for path in paths {
        let dir_path = path.expect("dirpath error").path();
        let _dir_name = dir_path.display();
        dirs.push(DiskEntry {
          path: format!("{}", _dir_name),
          is_dir: true,
          name: get_dir_name_from_path(_dir_name.to_string()),
        });
      }
      Ok(dirs)
    })
}

pub fn with_temp_dir<F: FnOnce(&tempfile::TempDir) -> ()>(callback: F) -> crate::Result<()> {
  let dir = tempdir()?;
  callback(&dir);
  dir.close()?;
  Ok(())
}

#[cfg(test)]
mod test {
  use super::*;
  use quickcheck_macros::quickcheck;
  use totems::assert_ok;

  // check is dir function by passing in arbitrary strings
  #[quickcheck]
  fn qc_is_dir(f: String) -> bool {
    // is the string runs through is_dir and comes out as an OK result then it must be a DIR.
    if let Ok(_) = is_dir(f.clone()) {
      std::path::PathBuf::from(f).exists()
    } else {
      false
    }
  }

  #[test]
  // check the walk_dir function
  fn check_walk_dir() {
    // define a relative directory string test/
    let dir = String::from("test/");
    // add the files to this directory
    let file_one = format!("{}test.txt", &dir).to_string();
    let file_two = format!("{}test_binary", &dir).to_string();

    // call walk_dir on the directory
    let res = walk_dir(dir.clone());

    // assert that the result is Ok()
    assert_ok!(&res);

    // destruct the OK into a vector of DiskEntry Structs
    if let Ok(vec) = res {
      // assert that the vector length is only 3
      assert_eq!(vec.len(), 3);

      // get the first DiskEntry
      let first = &vec[0];
      // get the second DiskEntry
      let second = &vec[1];
      // get the third DiskEntry
      let third = &vec[2];

      // check the fields for the first DiskEntry
      assert_eq!(first.path, dir);
      assert_eq!(first.is_dir, true);
      assert_eq!(first.name, dir);

      if second.path.contains(".txt") {
        // check the fields for the second DiskEntry
        assert_eq!(second.path, file_one);
        assert_eq!(second.is_dir, false);
        assert_eq!(second.name, file_one);

        // check the fields for the third DiskEntry
        assert_eq!(third.path, file_two);
        assert_eq!(third.is_dir, false);
        assert_eq!(third.name, file_two);
      } else {
        // check the fields for the second DiskEntry
        assert_eq!(second.path, file_two);
        assert_eq!(second.is_dir, false);
        assert_eq!(second.name, file_two);

        // check the fields for the third DiskEntry
        assert_eq!(third.path, file_one);
        assert_eq!(third.is_dir, false);
        assert_eq!(third.name, file_one);
      }
    }
  }

  #[test]
  // check the list_dir_contents function
  fn check_list_dir_contents() {
    // define a relative directory string test/
    let dir = String::from("test/");

    // call list_dir_contents on the dir string
    let res = list_dir_contents(&dir);

    // assert that the result is Ok()
    assert_ok!(&res);

    // destruct the vector from the Ok()
    if let Ok(vec) = res {
      // assert the length of the vector is 2
      assert_eq!(vec.len(), 2);

      // get the two DiskEntry structs in this vector
      let first = &vec[0];
      let second = &vec[1];

      if first.path.contains(".txt") {
        // check the fields for the first DiskEntry
        assert_eq!(first.path, "test/test.txt".to_string());
        assert_eq!(first.is_dir, true);
        assert_eq!(first.name, "test.txt".to_string());

        // check the fields for the second DiskEntry
        assert_eq!(second.path, "test/test_binary".to_string());
        assert_eq!(second.is_dir, true);
        assert_eq!(second.name, "test_binary".to_string());
      } else {
        // check the fields for the first DiskEntry
        assert_eq!(second.path, "test/test.txt".to_string());
        assert_eq!(second.is_dir, true);
        assert_eq!(second.name, "test.txt".to_string());

        // check the fields for the second DiskEntry
        assert_eq!(first.path, "test/test_binary".to_string());
        assert_eq!(first.is_dir, true);
        assert_eq!(first.name, "test_binary".to_string());
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
    assert_ok!(res);
  }
}
