// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  api::{dir, file, path::BaseDirectory},
  scope::Scopes,
  Config, Env, Manager, PackageInfo, Runtime, Window,
};

use super::InvokeContext;
use serde::{Deserialize, Serialize};
use tauri_macros::{module_command_handler, CommandModule};

use std::{
  fs,
  fs::File,
  io::Write,
  path::{Path, PathBuf},
  sync::Arc,
};

/// The options for the directory functions on the file system API.
#[derive(Debug, Clone, Deserialize)]
pub struct DirOperationOptions {
  /// Whether the API should recursively perform the operation on the directory.
  #[serde(default)]
  pub recursive: bool,
  /// The base directory of the operation.
  /// The directory path of the BaseDirectory will be the prefix of the defined directory path.
  pub dir: Option<BaseDirectory>,
}

/// The options for the file functions on the file system API.
#[derive(Debug, Clone, Deserialize)]
pub struct FileOperationOptions {
  /// The base directory of the operation.
  /// The directory path of the BaseDirectory will be the prefix of the defined file path.
  pub dir: Option<BaseDirectory>,
}

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
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
}

impl Cmd {
  #[module_command_handler(fs_read_text_file, "fs > readTextFile")]
  fn read_text_file<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<String> {
    file::read_string(resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.and_then(|o| o.dir),
    )?)
    .map_err(crate::Error::FailedToExecuteApi)
  }

  #[module_command_handler(fs_read_binary_file, "fs > readBinaryFile")]
  fn read_binary_file<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<Vec<u8>> {
    file::read_binary(resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.and_then(|o| o.dir),
    )?)
    .map_err(crate::Error::FailedToExecuteApi)
  }

  #[module_command_handler(fs_write_file, "fs > writeFile")]
  fn write_file<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    contents: String,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<()> {
    File::create(resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.and_then(|o| o.dir),
    )?)
    .map_err(crate::Error::Io)
    .and_then(|mut f| f.write_all(contents.as_bytes()).map_err(|err| err.into()))?;
    Ok(())
  }

  #[module_command_handler(fs_write_binary_file, "fs > writeBinaryFile")]
  fn write_binary_file<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    contents: String,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<()> {
    base64::decode(contents)
      .map_err(crate::Error::Base64Decode)
      .and_then(|c| {
        File::create(resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          path,
          options.and_then(|o| o.dir),
        )?)
        .map_err(Into::into)
        .and_then(|mut f| f.write_all(&c).map_err(|err| err.into()))
      })?;
    Ok(())
  }

  #[module_command_handler(fs_read_dir, "fs > readDir")]
  fn read_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    options: Option<DirOperationOptions>,
  ) -> crate::Result<Vec<dir::DiskEntry>> {
    let (recursive, dir) = if let Some(options_value) = options {
      (options_value.recursive, options_value.dir)
    } else {
      (false, None)
    };
    dir::read_dir(
      resolve_path(
        &context.config,
        &context.package_info,
        &context.window,
        path,
        dir,
      )?,
      recursive,
    )
    .map_err(crate::Error::FailedToExecuteApi)
  }

  #[module_command_handler(fs_copy_file, "fs > copyFile")]
  fn copy_file<R: Runtime>(
    context: InvokeContext<R>,
    source: PathBuf,
    destination: PathBuf,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<()> {
    let (src, dest) = match options.and_then(|o| o.dir) {
      Some(dir) => (
        resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          source,
          Some(dir.clone()),
        )?,
        resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          destination,
          Some(dir),
        )?,
      ),
      None => (source, destination),
    };
    fs::copy(src, dest)?;
    Ok(())
  }

  #[module_command_handler(fs_create_dir, "fs > createDir")]
  fn create_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    options: Option<DirOperationOptions>,
  ) -> crate::Result<()> {
    let (recursive, dir) = if let Some(options_value) = options {
      (options_value.recursive, options_value.dir)
    } else {
      (false, None)
    };
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      dir,
    )?;
    if recursive {
      fs::create_dir_all(resolved_path)?;
    } else {
      fs::create_dir(resolved_path)?;
    }

    Ok(())
  }

  #[module_command_handler(fs_remove_dir, "fs > removeDir")]
  fn remove_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    options: Option<DirOperationOptions>,
  ) -> crate::Result<()> {
    let (recursive, dir) = if let Some(options_value) = options {
      (options_value.recursive, options_value.dir)
    } else {
      (false, None)
    };
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      dir,
    )?;
    if recursive {
      fs::remove_dir_all(resolved_path)?;
    } else {
      fs::remove_dir(resolved_path)?;
    }

    Ok(())
  }

  #[module_command_handler(fs_remove_file, "fs > removeFile")]
  fn remove_file<R: Runtime>(
    context: InvokeContext<R>,
    path: PathBuf,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<()> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.and_then(|o| o.dir),
    )?;
    fs::remove_file(resolved_path)?;
    Ok(())
  }

  #[module_command_handler(fs_rename_file, "fs > renameFile")]
  fn rename_file<R: Runtime>(
    context: InvokeContext<R>,
    old_path: PathBuf,
    new_path: PathBuf,
    options: Option<FileOperationOptions>,
  ) -> crate::Result<()> {
    let (old, new) = match options.and_then(|o| o.dir) {
      Some(dir) => (
        resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          old_path,
          Some(dir.clone()),
        )?,
        resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          new_path,
          Some(dir),
        )?,
      ),
      None => (old_path, new_path),
    };
    fs::rename(old, new).map_err(crate::Error::Io)
  }
}

#[allow(dead_code)]
fn resolve_path<R: Runtime, P: AsRef<Path>>(
  config: &Config,
  package_info: &PackageInfo,
  window: &Window<R>,
  path: P,
  dir: Option<BaseDirectory>,
) -> crate::Result<PathBuf> {
  let env = window.state::<Env>().inner();
  match crate::api::path::resolve_path(config, package_info, env, path, dir) {
    Ok(path) => {
      if window.state::<Scopes>().fs.is_allowed(&path) {
        Ok(path)
      } else {
        Err(crate::Error::PathNotAllowed(path))
      }
    }
    Err(e) => Err(e.into()),
  }
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use super::{BaseDirectory, DirOperationOptions, FileOperationOptions};
  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for BaseDirectory {
    fn arbitrary(g: &mut Gen) -> Self {
      if bool::arbitrary(g) {
        BaseDirectory::App
      } else {
        BaseDirectory::Resource
      }
    }
  }

  impl Arbitrary for FileOperationOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for DirOperationOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        recursive: bool::arbitrary(g),
        dir: Option::arbitrary(g),
      }
    }
  }

  #[tauri_macros::module_command_test(fs_read_text_file, "fs > readTextFile")]
  #[quickcheck_macros::quickcheck]
  fn read_text_file(path: PathBuf, options: Option<FileOperationOptions>) {
    let res = super::Cmd::read_text_file(crate::test::mock_invoke_context(), path, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_read_binary_file, "fs > readBinaryFile")]
  #[quickcheck_macros::quickcheck]
  fn read_binary_file(path: PathBuf, options: Option<FileOperationOptions>) {
    let res = super::Cmd::read_binary_file(crate::test::mock_invoke_context(), path, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_write_file, "fs > writeFile")]
  #[quickcheck_macros::quickcheck]
  fn write_file(path: PathBuf, contents: String, options: Option<FileOperationOptions>) {
    let res = super::Cmd::write_file(crate::test::mock_invoke_context(), path, contents, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_read_binary_file, "fs > writeBinaryFile")]
  #[quickcheck_macros::quickcheck]
  fn write_binary_file(path: PathBuf, contents: String, options: Option<FileOperationOptions>) {
    let res =
      super::Cmd::write_binary_file(crate::test::mock_invoke_context(), path, contents, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_read_dir, "fs > readDir")]
  #[quickcheck_macros::quickcheck]
  fn read_dir(path: PathBuf, options: Option<DirOperationOptions>) {
    let res = super::Cmd::read_dir(crate::test::mock_invoke_context(), path, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_copy_file, "fs > copyFile")]
  #[quickcheck_macros::quickcheck]
  fn copy_file(source: PathBuf, destination: PathBuf, options: Option<FileOperationOptions>) {
    let res = super::Cmd::copy_file(
      crate::test::mock_invoke_context(),
      source,
      destination,
      options,
    );
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_create_dir, "fs > createDir")]
  #[quickcheck_macros::quickcheck]
  fn create_dir(path: PathBuf, options: Option<DirOperationOptions>) {
    let res = super::Cmd::create_dir(crate::test::mock_invoke_context(), path, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_remove_dir, "fs > removeDir")]
  #[quickcheck_macros::quickcheck]
  fn remove_dir(path: PathBuf, options: Option<DirOperationOptions>) {
    let res = super::Cmd::remove_dir(crate::test::mock_invoke_context(), path, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_remove_file, "fs > removeFile")]
  #[quickcheck_macros::quickcheck]
  fn remove_file(path: PathBuf, options: Option<FileOperationOptions>) {
    let res = super::Cmd::remove_file(crate::test::mock_invoke_context(), path, options);
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }

  #[tauri_macros::module_command_test(fs_rename_file, "fs > renameFile")]
  #[quickcheck_macros::quickcheck]
  fn rename_file(old_path: PathBuf, new_path: PathBuf, options: Option<FileOperationOptions>) {
    let res = super::Cmd::rename_file(
      crate::test::mock_invoke_context(),
      old_path,
      new_path,
      options,
    );
    assert!(!matches!(res, Err(crate::Error::ApiNotAllowlisted(_))));
  }
}
