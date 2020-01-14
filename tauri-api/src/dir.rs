use tempfile;

mod utils;
use ignore::Walk;
use serde::Serialize;
use std::fs;
use std::fs::metadata;
use utils::get_dir_name_from_path;

use tempfile::tempdir;

#[derive(Serialize)]
pub struct DiskEntry {
  pub path: String,
  pub is_dir: bool,
  pub name: String,
}

fn is_dir(file_name: String) -> crate::Result<bool> {
  match metadata(file_name.to_string()) {
    Ok(md) => return Result::Ok(md.is_dir()),
    Err(err) => return Result::Err(err.to_string().into()),
  };
}

pub fn walk_dir(path_copy: String) -> crate::Result<Vec<DiskEntry>> {
  println!("Trying to walk: {}", path_copy.as_str());
  let mut files_and_dirs: Vec<DiskEntry> = vec![];
  for result in Walk::new(path_copy) {
    match result {
      Ok(entry) => {
        let display_value = entry.path().display();
        let _dir_name = display_value.to_string();

        match is_dir(display_value.to_string()) {
          Ok(flag) => {
            files_and_dirs.push(DiskEntry {
              path: display_value.to_string(),
              is_dir: flag,
              name: display_value.to_string(),
            });
          }
          Err(_) => {}
        }
      }
      Err(_) => {}
    }
  }
  return Result::Ok(files_and_dirs);
}

pub fn list_dir_contents(dir_path: &String) -> crate::Result<Vec<DiskEntry>> {
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
  use crate::dir::*;

  // check is dir function by passing in arbitrary strings
  #[quickcheck]
  fn qc_is_dir(f: String) -> bool {
    // is the string runs through is_dir and comes out as an OK result then it must be a DIR.
    match is_dir(f.clone()) {
      // check to see that the path exists.
      Ok(_) => std::path::PathBuf::from(f).exists(),
      // if is Err then string isn't a path nor a dir and function passes.
      Err(_) => true,
    }
  }
}
