// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;
use crate::{
  api::{
    dir, file,
    path::{resolve_path, BaseDirectory},
  },
  Config,
};

use serde::{Deserialize, Serialize};

use std::{fs, fs::File, io::Write, path::PathBuf, sync::Arc};

/// The options for the directory functions on the file system API.
#[derive(Deserialize)]
pub struct DirOperationOptions {
  /// Whether the API should recursively perform the operation on the directory.
  #[serde(default)]
  pub recursive: bool,
  /// The base directory of the operation.
  /// The directory path of the BaseDirectory will be the prefix of the defined directory path.
  pub dir: Option<BaseDirectory>,
}

/// The options for the file functions on the file system API.
#[derive(Deserialize)]
pub struct FileOperationOptions {
  /// The base directory of the operation.
  /// The directory path of the BaseDirectory will be the prefix of the defined file path.
  pub dir: Option<BaseDirectory>,
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The read text file API.
  ReadTextFile {
    path: PathBuf,
    options: Option<FileOperationOptions>,
  },
  /// The read binary file API.
  ReadBinaryFile {
    path: PathBuf,
    options: Option<FileOperationOptions>,
  },
  /// The write file API.
  WriteFile {
    path: PathBuf,
    contents: String,
    options: Option<FileOperationOptions>,
  },
  /// The write binary file API.
  WriteBinaryFile {
    path: PathBuf,
    contents: String,
    options: Option<FileOperationOptions>,
  },
  /// The read dir API.
  ReadDir {
    path: PathBuf,
    options: Option<DirOperationOptions>,
  },
  /// The copy file API.
  CopyFile {
    source: PathBuf,
    destination: PathBuf,
    options: Option<FileOperationOptions>,
  },
  /// The create dir API.
  CreateDir {
    path: PathBuf,
    options: Option<DirOperationOptions>,
  },
  /// The remove dir API.
  RemoveDir {
    path: PathBuf,
    options: Option<DirOperationOptions>,
  },
  /// The remove file API.
  RemoveFile {
    path: PathBuf,
    options: Option<FileOperationOptions>,
  },
  /// The rename file API.
  #[serde(rename_all = "camelCase")]
  RenameFile {
    old_path: PathBuf,
    new_path: PathBuf,
    options: Option<FileOperationOptions>,
  },
  /// The resolve path API
  ResolvePath {
    path: String,
    directory: Option<BaseDirectory>,
  },
}

impl Cmd {
  pub fn run(self, config: Arc<Config>) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(fs_read_text_file)]
      Self::ReadTextFile { path, options } => read_text_file(&config, path, options).map(Into::into),
      #[cfg(not(fs_read_text_file))]
      Self::ReadTextFile { .. } => Err(crate::Error::ApiNotAllowlisted(
        "fs > readTextFile".to_string(),
      )),

      #[cfg(fs_read_binary_file)]
      Self::ReadBinaryFile { path, options } => read_binary_file(&config, path, options).map(Into::into),
      #[cfg(not(fs_read_binary_file))]
      Self::ReadBinaryFile { .. } => Err(crate::Error::ApiNotAllowlisted(
        "readBinaryFile".to_string(),
      )),

      #[cfg(fs_write_file)]
      Self::WriteFile {
        path,
        contents,
        options,
      } => write_file(&config, path, contents, options).map(Into::into),
      #[cfg(not(fs_write_file))]
      Self::WriteFile { .. } => Err(crate::Error::ApiNotAllowlisted(
        "fs > writeFile".to_string(),
      )),

      #[cfg(fs_write_binary_file)]
      Self::WriteBinaryFile {
        path,
        contents,
        options,
      } => write_binary_file(&config, path, contents, options).map(Into::into),
      #[cfg(not(fs_write_binary_file))]
      Self::WriteBinaryFile { .. } => Err(crate::Error::ApiNotAllowlisted(
        "writeBinaryFile".to_string(),
      )),

      #[cfg(fs_read_dir)]
      Self::ReadDir { path, options } => read_dir(&config, path, options).map(Into::into),
      #[cfg(not(fs_read_dir))]
      Self::ReadDir { .. } => Err(crate::Error::ApiNotAllowlisted("fs > readDir".to_string())),

      #[cfg(fs_copy_file)]
      Self::CopyFile {
        source,
        destination,
        options,
      } => copy_file(&config, source, destination, options).map(Into::into),
      #[cfg(not(fs_copy_file))]
      Self::CopyFile { .. } => Err(crate::Error::ApiNotAllowlisted("fs > copyFile".to_string())),

      #[cfg(fs_create_dir)]
      Self::CreateDir { path, options } => create_dir(&config, path, options).map(Into::into),
      #[cfg(not(fs_create_dir))]
      Self::CreateDir { .. } => Err(crate::Error::ApiNotAllowlisted(
        "fs > createDir".to_string(),
      )),

      #[cfg(fs_remove_dir)]
      Self::RemoveDir { path, options } => remove_dir(&config, path, options).map(Into::into),
      #[cfg(not(fs_remove_dir))]
      Self::RemoveDir { .. } => Err(crate::Error::ApiNotAllowlisted(
        "fs > removeDir".to_string(),
      )),

      #[cfg(fs_remove_file)]
      Self::RemoveFile { path, options } => remove_file(&config, path, options).map(Into::into),
      #[cfg(not(fs_remove_file))]
      Self::RemoveFile { .. } => Err(crate::Error::ApiNotAllowlisted(
        "fs > removeFile".to_string(),
      )),

      #[cfg(fs_rename_file)]
      Self::RenameFile {
        old_path,
        new_path,
        options,
      } => rename_file(&config, old_path, new_path, options).map(Into::into),
      #[cfg(not(fs_rename_file))]
      Self::RenameFile { .. } => Err(crate::Error::ApiNotAllowlisted(
        "fs > renameFile".to_string(),
      )),

      #[cfg(fs_path)]
      Self::ResolvePath { path, directory } => {
        resolve_path_handler(&config, path, directory).map(Into::into)
      }
      #[cfg(not(fs_path))]
      Self::ResolvePath { .. } => Err(crate::Error::ApiNotAllowlisted("fs > pathApi".to_string())),
    }
  }
}

/// Reads a directory.
#[cfg(fs_read_dir)]
pub fn read_dir(
  config: &Config,
  path: PathBuf,
  options: Option<DirOperationOptions>,
) -> crate::Result<Vec<dir::DiskEntry>> {
  let (recursive, dir) = if let Some(options_value) = options {
    (options_value.recursive, options_value.dir)
  } else {
    (false, None)
  };
  dir::read_dir(resolve_path(config, path, dir)?, recursive)
    .map_err(crate::Error::FailedToExecuteApi)
}

/// Copies a file.
#[cfg(fs_copy_file)]
pub fn copy_file(
  config: &Config,
  source: PathBuf,
  destination: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  let (src, dest) = match options.and_then(|o| o.dir) {
    Some(dir) => (
      resolve_path(config, source, Some(dir.clone()))?,
      resolve_path(config, destination, Some(dir))?,
    ),
    None => (source, destination),
  };
  fs::copy(src, dest)?;
  Ok(())
}

/// Creates a directory.
#[cfg(fs_create_dir)]
pub fn create_dir(
  config: &Config,
  path: PathBuf,
  options: Option<DirOperationOptions>,
) -> crate::Result<()> {
  let (recursive, dir) = if let Some(options_value) = options {
    (options_value.recursive, options_value.dir)
  } else {
    (false, None)
  };
  let resolved_path = resolve_path(config, path, dir)?;
  if recursive {
    fs::create_dir_all(resolved_path)?;
  } else {
    fs::create_dir(resolved_path)?;
  }

  Ok(())
}

/// Removes a directory.
#[cfg(fs_remove_dir)]
pub fn remove_dir(
  config: &Config,
  path: PathBuf,
  options: Option<DirOperationOptions>,
) -> crate::Result<()> {
  let (recursive, dir) = if let Some(options_value) = options {
    (options_value.recursive, options_value.dir)
  } else {
    (false, None)
  };
  let resolved_path = resolve_path(config, path, dir)?;
  if recursive {
    fs::remove_dir_all(resolved_path)?;
  } else {
    fs::remove_dir(resolved_path)?;
  }

  Ok(())
}

/// Removes a file
#[cfg(fs_remove_file)]
pub fn remove_file(
  config: &Config,
  path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  let resolved_path = resolve_path(config, path, options.and_then(|o| o.dir))?;
  fs::remove_file(resolved_path)?;
  Ok(())
}

/// Renames a file.
#[cfg(fs_rename_file)]
pub fn rename_file(
  config: &Config,
  old_path: PathBuf,
  new_path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  let (old, new) = match options.and_then(|o| o.dir) {
    Some(dir) => (
      resolve_path(config, old_path, Some(dir.clone()))?,
      resolve_path(config, new_path, Some(dir))?,
    ),
    None => (old_path, new_path),
  };
  fs::rename(old, new).map_err(crate::Error::Io)
}

/// Writes a text file.
#[cfg(fs_write_file)]
pub fn write_file(
  config: &Config,
  path: PathBuf,
  contents: String,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  File::create(resolve_path(config, path, options.and_then(|o| o.dir))?)
    .map_err(crate::Error::Io)
    .and_then(|mut f| f.write_all(contents.as_bytes()).map_err(|err| err.into()))?;
  Ok(())
}

/// Writes a binary file.
#[cfg(fs_write_binary_file)]
pub fn write_binary_file(
  config: &Config,
  path: PathBuf,
  contents: String,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  base64::decode(contents)
    .map_err(crate::Error::Base64Decode)
    .and_then(|c| {
      File::create(resolve_path(config, path, options.and_then(|o| o.dir))?)
        .map_err(Into::into)
        .and_then(|mut f| f.write_all(&c).map_err(|err| err.into()))
    })?;
  Ok(())
}

/// Reads a text file.
#[cfg(fs_read_text_file)]
pub fn read_text_file(
  config: &Config,
  path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<String> {
  file::read_string(resolve_path(config, path, options.and_then(|o| o.dir))?)
    .map_err(crate::Error::FailedToExecuteApi)
}

/// Reads a binary file.
#[cfg(fs_read_binary_file)]
pub fn read_binary_file(
  config: &Config,
  path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<Vec<u8>> {
  file::read_binary(resolve_path(config, path, options.and_then(|o| o.dir))?)
    .map_err(crate::Error::FailedToExecuteApi)
}

#[cfg(fs_path)]
pub fn resolve_path_handler(
  config: &Config,
  path: String,
  directory: Option<BaseDirectory>,
) -> crate::Result<PathBuf> {
  resolve_path(config, path, directory).map_err(Into::into)
}

// test webview functionality.
#[cfg(test)]
mod test {
  // use super::*;
  // use web_view::*;

  // create a makeshift webview
  // fn create_test_webview() -> crate::Result<WebView<'static, ()>> {
  //   // basic html set into webview
  //   let content = r#"<html><head></head><body></body></html>"#;

  //   Ok(
  //     // use webview builder to create simple webview
  //     WebViewBuilder::new()
  //       .title("test")
  //       .size(800, 800)
  //       .resizable(true)
  //       .debug(true)
  //       .user_data(())
  //       .invoke_handler(|_wv, _command, _arg| Ok(()))
  //       .content(Content::Html(content))
  //       .build()?,
  //   )
  // }

  /* #[test]
  #[cfg(not(any(target_os = "linux", target_os = "macos")))]
  // test the file_write functionality
  fn test_write_to_file() -> crate::Result<()> {
    // import read_to_string and write to be able to manipulate the file.
    use std::fs::{read_to_string, write};

    // create the webview
    let mut webview = create_test_webview()?;

    // setup the contents and the path.
    let contents = String::from(r#"Write to the Test file"#);
    let path = String::from("test/fixture/test.txt".to_string()));

    // clear the file by writing nothing to it.
    write(&path, "")?;

    //call write file with the path and contents.
    write_file(
      &webview_manager,
      path.clone(),
      contents.clone(),
      String::from(""),
      String::from(""),
    );

    // sleep the main thread to wait for the promise to execute.
    std::thread::sleep(std::time::Duration::from_millis(200));

    // read from the file.
    let data = read_to_string(path)?;

    // check that the file contents is equal to the expected contents.
    assert_eq!(data, contents);

    Ok(())
  } */
}
