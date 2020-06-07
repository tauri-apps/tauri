use std::fs::{create_dir, create_dir_all, read_dir, remove_dir_all};
use std::path::{Path, PathBuf};

#[derive(Clone)]
pub struct DirOpts {
  pub depth: u64,
}

pub struct FileOpts {
  pub overwrite: bool,
  pub skip: bool,
  pub buffer_size: usize,
}

#[derive(Clone)]
pub struct Options {
  pub overwrite: bool,
  pub skip: bool,
  pub buffer_size: usize,
  pub copy_files: bool,
  pub content_only: bool,
  pub depth: u64,
}

pub struct DirInfo {
  pub size: u64,
  pub files: Vec<String>,
  pub directories: Vec<String>,
}

impl Default for Options {
  fn default() -> Options {
    Options {
      overwrite: false,
      skip: false,
      buffer_size: 64000,
      copy_files: false,
      content_only: false,
      depth: 0,
    }
  }
}

impl Default for DirOpts {
  fn default() -> DirOpts {
    DirOpts { depth: 0 }
  }
}

impl Default for FileOpts {
  fn default() -> FileOpts {
    FileOpts {
      overwrite: false,
      skip: false,
      buffer_size: 64000,
    }
  }
}

pub fn create<P>(path: P, erase: bool) -> crate::Result<()>
where
  P: AsRef<Path>,
{
  if erase && path.as_ref().exists() {
    remove(&path)?;
  }
  Ok(create_dir(&path)?)
}

pub fn create_all<P>(path: P, erase: bool) -> crate::Result<()>
where
  P: AsRef<Path>,
{
  if erase && path.as_ref().exists() {
    remove(&path)?;
  }
  Ok(create_dir_all(&path)?)
}

pub fn remove<P: AsRef<Path>>(path: P) -> crate::Result<()> {
  if path.as_ref().exists() {
    Ok(remove_dir_all(path)?)
  } else {
    Ok(())
  }
}

pub fn copy_file<P, Q>(from: P, to: Q, options: &FileOpts) -> crate::Result<u64>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let from = from.as_ref();
  if !from.exists() {
    if let Some(msg) = from.to_str() {
      let msg = format!("Path \"{}\" does not exist or you don't have access", msg);
      return Err(crate::Error::PathUtilError(msg));
    }
    return Err(crate::Error::PathUtilError(
      "Path does not exist or you don't have access!".to_owned(),
    ));
  }

  if !from.is_file() {
    if let Some(msg) = from.to_str() {
      let msg = format!("Path \"{}\" is not a file!", msg);
      return Err(crate::Error::PathUtilError(msg));
    }
    return Err(crate::Error::PathUtilError(
      "Path is not a file!".to_owned(),
    ));
  }
  if !options.overwrite && to.as_ref().exists() {
    if options.skip {
      return Ok(0);
    }

    if let Some(msg) = to.as_ref().to_str() {
      let msg = format!("Path \"{}\" is exist", msg);
      return Err(crate::Error::PathUtilError(msg));
    }
  }

  Ok(std::fs::copy(from, to)?)
}

#[allow(dead_code)]
pub fn copy<P, Q>(from: P, to: Q, options: &Options) -> crate::Result<u64>
where
  P: AsRef<Path>,
  Q: AsRef<Path>,
{
  let from = from.as_ref();
  if !from.exists() {
    if let Some(msg) = from.to_str() {
      let msg = format!("Path \"{}\" does not exist or you don't have access!", msg);
      return Err(crate::Error::PathUtilError(msg));
    }
    return Err(crate::Error::PathUtilError(
      "Path does not exist or you don't have access".to_owned(),
    ));
  }
  if !from.is_dir() {
    if let Some(msg) = from.to_str() {
      let msg = format!("Path \"{}\" is not a directory!", msg);
      return Err(crate::Error::PathUtilError(msg));
    }
    return Err(crate::Error::PathUtilError(
      "Path is not a directory".to_owned(),
    ));
  }
  let dir_name;
  if let Some(val) = from.components().last() {
    dir_name = val.as_os_str();
  } else {
    return Err(crate::Error::PathUtilError(
      "Invalid Folder form".to_owned(),
    ));
  }
  let mut to: PathBuf = to.as_ref().to_path_buf();
  if !options.content_only && (!options.copy_files || to.exists()) {
    to.push(dir_name);
  }

  let mut read_options = DirOpts::default();
  if options.depth > 0 {
    read_options.depth = options.depth;
  }

  let dir_content = get_dir_info(from, &read_options)?;
  for directory in dir_content.directories {
    let tmp_to = Path::new(&directory).strip_prefix(from)?;
    let dir = to.join(&tmp_to);
    if !dir.exists() {
      if options.copy_files {
        create_all(dir, false)?;
      } else {
        create(dir, false)?;
      }
    }
  }
  let mut result: u64 = 0;
  for file in dir_content.files {
    let to = to.to_path_buf();
    let tp = Path::new(&file).strip_prefix(from)?;
    let path = to.join(&tp);

    let file_options = FileOpts {
      overwrite: options.overwrite,
      skip: options.skip,
      buffer_size: options.buffer_size,
    };
    let mut result_copy: crate::Result<u64>;
    let mut work = true;

    while work {
      result_copy = copy_file(&file, &path, &file_options);
      match result_copy {
        Ok(val) => {
          result += val;
          work = false;
        }
        Err(err) => {
          let err_msg = err.to_string();
          return Err(crate::Error::PathUtilError(err_msg));
        }
      }
    }
  }
  Ok(result)
}

pub fn get_dir_info<P>(path: P, options: &DirOpts) -> crate::Result<DirInfo>
where
  P: AsRef<Path>,
{
  let depth = if options.depth == 0 {
    0
  } else {
    options.depth + 1
  };

  _get_dir_info(path, depth)
}

fn _get_dir_info<P>(path: P, mut depth: u64) -> crate::Result<DirInfo>
where
  P: AsRef<Path>,
{
  let mut directories = Vec::new();
  let mut files = Vec::new();
  let mut size = 0;
  let item = path.as_ref().to_str();
  if item.is_none() {
    return Err(crate::Error::PathUtilError("Invalid Path".to_owned()));
  }
  let item = item.expect("Item had no data").to_string();

  if path.as_ref().is_dir() {
    directories.push(item);
    if depth == 0 || depth > 1 {
      if depth > 1 {
        depth -= 1;
      }
      for entry in read_dir(&path)? {
        let _path = entry?.path();

        match _get_dir_info(_path, depth) {
          Ok(items) => {
            let mut _files = items.files;
            let mut _directories = items.directories;
            size += items.size;
            files.append(&mut _files);
            directories.append(&mut _directories);
          }
          Err(err) => return Err(err),
        }
      }
    }
  } else {
    size = path.as_ref().metadata()?.len();
    files.push(item);
  }
  Ok(DirInfo {
    size,
    files,
    directories,
  })
}
