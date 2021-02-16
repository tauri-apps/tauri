use crate::{api::path::BaseDirectory, ApplicationDispatcherExt};

use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use tauri_api::{dir, file, path::resolve_path};

use std::{fs, fs::File, io::Write, path::PathBuf};

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
  pub async fn run(self) -> crate::Result<JsonValue> {
    match self {
      Self::ReadTextFile { path, options } => {
        #[cfg(read_text_file)]
        return read_text_file(path, options)
          .await
          .and_then(super::to_value);
        #[cfg(not(read_text_file))]
        Err(crate::Error::ApiNotAllowlisted("readTextFile".to_string()))
      }
      Self::ReadBinaryFile { path, options } => {
        #[cfg(read_binary_file)]
        return read_binary_file(path, options)
          .await
          .and_then(super::to_value);
        #[cfg(not(read_binary_file))]
        Err(crate::Error::ApiNotAllowlisted(
          "readBinaryFile".to_string(),
        ))
      }
      Self::WriteFile {
        path,
        contents,
        options,
      } => {
        #[cfg(write_file)]
        return write_file(path, contents, options)
          .await
          .and_then(super::to_value);
        #[cfg(not(write_file))]
        Err(crate::Error::ApiNotAllowlisted("writeFile".to_string()))
      }
      Self::WriteBinaryFile {
        path,
        contents,
        options,
      } => {
        #[cfg(write_binary_file)]
        return write_binary_file(path, contents, options)
          .await
          .and_then(super::to_value);
        #[cfg(not(write_binary_file))]
        Err(crate::Error::ApiNotAllowlisted(
          "writeBinaryFile".to_string(),
        ))
      }
      Self::ReadDir { path, options } => {
        #[cfg(read_dir)]
        return read_dir(path, options).await.and_then(super::to_value);
        #[cfg(not(read_dir))]
        Err(crate::Error::ApiNotAllowlisted("readDir".to_string()))
      }
      Self::CopyFile {
        source,
        destination,
        options,
      } => {
        #[cfg(copy_file)]
        return copy_file(source, destination, options)
          .await
          .and_then(super::to_value);
        #[cfg(not(copy_file))]
        Err(crate::Error::ApiNotAllowlisted("copyFile".to_string()))
      }
      Self::CreateDir { path, options } => {
        #[cfg(create_dir)]
        return create_dir(path, options).await.and_then(super::to_value);
        #[cfg(not(create_dir))]
        Err(crate::Error::ApiNotAllowlisted("createDir".to_string()))
      }
      Self::RemoveDir { path, options } => {
        #[cfg(remove_dir)]
        return remove_dir(path, options).await.and_then(super::to_value);
        #[cfg(not(remove_dir))]
        Err(crate::Error::ApiNotAllowlisted("removeDir".to_string()))
      }
      Self::RemoveFile { path, options } => {
        #[cfg(remove_file)]
        return remove_file(path, options).await.and_then(super::to_value);
        #[cfg(not(remove_file))]
        Err(crate::Error::ApiNotAllowlisted("removeFile".to_string()))
      }
      Self::RenameFile {
        old_path,
        new_path,
        options,
      } => {
        #[cfg(rename_file)]
        return rename_file(old_path, new_path, options)
          .await
          .and_then(super::to_value);
        #[cfg(not(rename_file))]
        Err(crate::Error::ApiNotAllowlisted("renameFile".to_string()))
      }
      Self::ResolvePath { path, directory } => {
        #[cfg(path_api)]
        return resolve_path_handler(path, directory)
          .await
          .and_then(super::to_value);
        #[cfg(not(path_api))]
        Err(crate::Error::ApiNotAllowlisted("pathApi".to_string()))
      }
    }
  }
}

/// Reads a directory.
#[cfg(read_dir)]
pub async fn read_dir(
  path: PathBuf,
  options: Option<DirOperationOptions>,
) -> crate::Result<Vec<dir::DiskEntry>> {
  let (recursive, dir) = if let Some(options_value) = options {
    (options_value.recursive, options_value.dir)
  } else {
    (false, None)
  };
  dir::read_dir(resolve_path(path, dir)?, recursive).map_err(crate::Error::FailedToExecuteApi)
}

/// Copies a file.
#[cfg(copy_file)]
pub async fn copy_file(
  source: PathBuf,
  destination: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  let (src, dest) = match options.and_then(|o| o.dir) {
    Some(dir) => (
      resolve_path(source, Some(dir.clone()))?,
      resolve_path(destination, Some(dir))?,
    ),
    None => (source, destination),
  };
  fs::copy(src, dest)?;
  Ok(())
}

/// Creates a directory.
#[cfg(create_dir)]
pub async fn create_dir(path: PathBuf, options: Option<DirOperationOptions>) -> crate::Result<()> {
  let (recursive, dir) = if let Some(options_value) = options {
    (options_value.recursive, options_value.dir)
  } else {
    (false, None)
  };
  let resolved_path = resolve_path(path, dir)?;
  if recursive {
    fs::create_dir_all(resolved_path)?;
  } else {
    fs::create_dir(resolved_path)?;
  }

  Ok(())
}

/// Removes a directory.
#[cfg(remove_dir)]
pub async fn remove_dir(path: PathBuf, options: Option<DirOperationOptions>) -> crate::Result<()> {
  let (recursive, dir) = if let Some(options_value) = options {
    (options_value.recursive, options_value.dir)
  } else {
    (false, None)
  };
  let resolved_path = resolve_path(path, dir)?;
  if recursive {
    fs::remove_dir_all(resolved_path)?;
  } else {
    fs::remove_dir(resolved_path)?;
  }

  Ok(())
}

/// Removes a file
#[cfg(remove_file)]
pub async fn remove_file(
  path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  let resolved_path = resolve_path(path, options.and_then(|o| o.dir))?;
  fs::remove_file(resolved_path)?;
  Ok(())
}

/// Renames a file.
#[cfg(rename_file)]
pub async fn rename_file(
  old_path: PathBuf,
  new_path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  let (old, new) = match options.and_then(|o| o.dir) {
    Some(dir) => (
      resolve_path(old_path, Some(dir.clone()))?,
      resolve_path(new_path, Some(dir))?,
    ),
    None => (old_path, new_path),
  };
  fs::rename(old, new).map_err(crate::Error::Io)
}

/// Writes a text file.
#[cfg(write_file)]
pub async fn write_file(
  path: PathBuf,
  contents: String,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  File::create(resolve_path(path, options.and_then(|o| o.dir))?)
    .map_err(crate::Error::Io)
    .and_then(|mut f| f.write_all(contents.as_bytes()).map_err(|err| err.into()))?;
  Ok(())
}

/// Writes a binary file.
#[cfg(write_binary_file)]
pub async fn write_binary_file(
  path: PathBuf,
  contents: String,
  options: Option<FileOperationOptions>,
) -> crate::Result<()> {
  base64::decode(contents)
    .map_err(crate::Error::Base64Decode)
    .and_then(|c| {
      File::create(resolve_path(path, options.and_then(|o| o.dir))?)
        .map_err(Into::into)
        .and_then(|mut f| f.write_all(&c).map_err(|err| err.into()))
    })?;
  Ok(())
}

/// Reads a text file.
#[cfg(read_text_file)]
pub async fn read_text_file(
  path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<String> {
  file::read_string(resolve_path(path, options.and_then(|o| o.dir))?)
    .map_err(crate::Error::FailedToExecuteApi)
}

/// Reads a binary file.
#[cfg(read_binary_file)]
pub async fn read_binary_file(
  path: PathBuf,
  options: Option<FileOperationOptions>,
) -> crate::Result<Vec<u8>> {
  file::read_binary(resolve_path(path, options.and_then(|o| o.dir))?)
    .map_err(crate::Error::FailedToExecuteApi)
}

pub async fn resolve_path_handler(
  path: String,
  directory: Option<BaseDirectory>,
) -> crate::Result<PathBuf> {
  resolve_path(path, directory).map_err(Into::into)
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
  //       .invoke_handler(|_wv, _arg| Ok(()))
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
