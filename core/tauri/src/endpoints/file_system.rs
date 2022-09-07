// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_imports)]

use crate::{
  api::{
    dir,
    file::{self, SafePathBuf},
    path::BaseDirectory,
  },
  error::into_anyhow,
  scope::Scopes,
  Config, Env, Manager, PackageInfo, Runtime, Window,
};

use super::InvokeContext;
#[allow(unused_imports)]
use anyhow::Context;
use serde::{
  de::{Deserializer, Error as DeError},
  Deserialize, Serialize,
};
use tauri_macros::{command_enum, module_command_handler, CommandModule};

use std::fmt::{Debug, Formatter};
use std::{
  fs,
  fs::File,
  io::Write,
  path::{Component, Path},
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

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileOptions {
  append: Option<bool>,
  create: Option<bool>,
  #[allow(unused)]
  mode: Option<u32>,
  #[allow(unused)]
  access_mode: Option<u32>,
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadFileOptions {
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyFileOptions {
  from_path_base_dir: Option<BaseDirectory>,
  to_path_base_dir: Option<BaseDirectory>,
}

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub(crate) enum Cmd {
  /// The read binary file API.
  #[cmd(fs_read_file, "fs > readFile")]
  ReadFile {
    path: SafePathBuf,
    options: Option<ReadFileOptions>,
  },
  /// The read binary file API.
  #[cmd(fs_read_file, "fs > readFile")]
  ReadTextFile {
    path: SafePathBuf,
    options: Option<ReadFileOptions>,
  },

  #[cmd(fs_write_file, "fs > writeFile")]
  WriteFile {
    path: SafePathBuf,
    data: Vec<u8>,
    options: Option<WriteFileOptions>,
  },
  /// The write file API.
  #[cmd(fs_write_file, "fs > writeFile")]
  WriteTextFile {
    path: SafePathBuf,
    data: String,
    options: Option<WriteFileOptions>,
  },
  /// The read dir API.
  #[cmd(fs_read_dir, "fs > readDir")]
  ReadDir {
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
  },
  /// The copy file API.
  #[cmd(fs_copy_file, "fs > copyFile")]
  #[serde(rename_all = "camelCase")]
  CopyFile {
    from_path: SafePathBuf,
    to_path: SafePathBuf,
    options: Option<CopyFileOptions>,
  },
  /// The create dir API.
  #[cmd(fs_create_dir, "fs > createDir")]
  CreateDir {
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
  },
  /// The remove dir API.
  #[cmd(fs_remove_dir, "fs > removeDir")]
  RemoveDir {
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
  },
  /// The remove file API.
  #[cmd(fs_remove_file, "fs > removeFile")]
  RemoveFile {
    path: SafePathBuf,
    options: Option<FileOperationOptions>,
  },
  /// The rename file API.
  #[cmd(fs_rename_file, "fs > renameFile")]
  #[serde(rename_all = "camelCase")]
  RenameFile {
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<FileOperationOptions>,
  },
}

impl Cmd {
  #[module_command_handler(fs_read_file)]
  fn read_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<ReadFileOptions>,
  ) -> super::Result<Vec<u8>> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;
    file::read_binary(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(Into::into)
  }

  #[module_command_handler(fs_read_file)]
  fn read_text_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<ReadFileOptions>,
  ) -> super::Result<String> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;
    file::read_string(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(Into::into)
  }

  #[module_command_handler(fs_write_file)]
  fn write_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    data: Vec<u8>,
    options: Option<WriteFileOptions>,
  ) -> super::Result<()> {
    write_file(context, path, &data, options)
  }

  #[module_command_handler(fs_write_file)]
  fn write_text_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    data: String,
    options: Option<WriteFileOptions>,
  ) -> super::Result<()> {
    write_file(context, path, data.as_bytes(), options)
  }

  #[module_command_handler(fs_read_dir)]
  fn read_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
  ) -> super::Result<Vec<dir::DiskEntry>> {
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
    dir::read_dir(&resolved_path, recursive)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(Into::into)
  }

  #[module_command_handler(fs_copy_file)]
  fn copy_file<R: Runtime>(
    context: InvokeContext<R>,
    from_path: SafePathBuf,
    to_path: SafePathBuf,
    options: Option<CopyFileOptions>,
  ) -> super::Result<()> {
    let from_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      file_url_to_safe_pathbuf(from_path)?,
      options.as_ref().and_then(|o| o.from_path_base_dir),
    )?;
    let to_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      file_url_to_safe_pathbuf(to_path)?,
      options.as_ref().and_then(|o| o.to_path_base_dir),
    )?;
    fs::copy(&from_path, &to_path).with_context(|| {
      format!(
        "fromPath: {}, toPath: {}",
        from_path.display(),
        to_path.display()
      )
    })?;
    Ok(())
  }

  #[module_command_handler(fs_create_dir)]
  fn create_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
  ) -> super::Result<()> {
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
      fs::create_dir_all(&resolved_path)
        .with_context(|| format!("path: {}", resolved_path.display()))?;
    } else {
      fs::create_dir(&resolved_path)
        .with_context(|| format!("path: {} (non recursive)", resolved_path.display()))?;
    }

    Ok(())
  }

  #[module_command_handler(fs_remove_dir)]
  fn remove_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<DirOperationOptions>,
  ) -> super::Result<()> {
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
      fs::remove_dir_all(&resolved_path)
        .with_context(|| format!("path: {}", resolved_path.display()))?;
    } else {
      fs::remove_dir(&resolved_path)
        .with_context(|| format!("path: {} (non recursive)", resolved_path.display()))?;
    }

    Ok(())
  }

  #[module_command_handler(fs_remove_file)]
  fn remove_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<FileOperationOptions>,
  ) -> super::Result<()> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.and_then(|o| o.dir),
    )?;
    fs::remove_file(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))?;
    Ok(())
  }

  #[module_command_handler(fs_rename_file)]
  fn rename_file<R: Runtime>(
    context: InvokeContext<R>,
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<FileOperationOptions>,
  ) -> super::Result<()> {
    let (old, new) = match options.and_then(|o| o.dir) {
      Some(dir) => (
        resolve_path(
          &context.config,
          &context.package_info,
          &context.window,
          old_path,
          Some(dir),
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
    fs::rename(&old, &new)
      .with_context(|| format!("old: {}, new: {}", old.display(), new.display()))
      .map_err(Into::into)
  }
}

#[allow(dead_code)]
fn resolve_path<R: Runtime>(
  config: &Config,
  package_info: &PackageInfo,
  window: &Window<R>,
  path: SafePathBuf,
  dir: Option<BaseDirectory>,
) -> super::Result<SafePathBuf> {
  let env = window.state::<Env>().inner();
  match crate::api::path::resolve_path(config, package_info, env, &path, dir) {
    Ok(path) => {
      if window.state::<Scopes>().fs.is_allowed(&path) {
        Ok(
          // safety: the path is resolved by Tauri so it is safe
          unsafe { SafePathBuf::new_unchecked(path) },
        )
      } else {
        Err(anyhow::anyhow!(
          crate::Error::PathNotAllowed(path).to_string()
        ))
      }
    }
    Err(e) => super::Result::<SafePathBuf>::Err(e.into())
      .with_context(|| format!("path: {}, base dir: {:?}", path.display(), dir)),
  }
}

fn file_url_to_safe_pathbuf(path: SafePathBuf) -> super::Result<SafePathBuf> {
  if path.as_ref().starts_with("file:") {
    SafePathBuf::new(
      url::Url::parse(&path.display().to_string())?
        .to_file_path()
        .map_err(|_| into_anyhow("Failed to get path from `file:` url"))?,
    )
    .map_err(into_anyhow)
  } else {
    Ok(path)
  }
}

fn write_file<R: Runtime>(
  context: InvokeContext<R>,
  path: SafePathBuf,
  data: &[u8],
  options: Option<WriteFileOptions>,
) -> super::Result<()> {
  let path = file_url_to_safe_pathbuf(path)?;

  let resolved_path = resolve_path(
    &context.config,
    &context.package_info,
    &context.window,
    path,
    options.as_ref().and_then(|o| o.base_dir),
  )?;

  let mut opts = fs::OpenOptions::new();
  opts.append(options.as_ref().map(|o| o.append.unwrap_or(false)).unwrap());
  opts.create(options.as_ref().map(|o| o.create.unwrap_or(true)).unwrap());

  #[cfg(windows)]
  {
    use std::os::windows::fs::OpenOptionsExt;
    if let Some(Some(access_mode)) = options.map(|o| o.access_mode) {
      opts.access_mode(access_mode);
    }
  }

  #[cfg(not(windows))]
  {
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(Some(mode)) = options.map(|o| o.mode) {
      opts.mode(mode);
    }
  }

  opts
    .write(true)
    .open(&resolved_path)
    .with_context(|| format!("path: {}", resolved_path.display()))
    .map_err(Into::into)
    .and_then(|mut f| f.write_all(data).map_err(|err| err.into()))
}

#[cfg(test)]
mod tests {
  use super::{
    BaseDirectory, CopyFileOptions, DirOperationOptions, FileOperationOptions, ReadFileOptions,
    SafePathBuf, WriteFileOptions,
  };

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

  impl Arbitrary for WriteFileOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        append: Option::arbitrary(g),
        create: Option::arbitrary(g),
        mode: Option::arbitrary(g),
        access_mode: Option::arbitrary(g),
        base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for ReadFileOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for CopyFileOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        from_path_base_dir: Option::arbitrary(g),
        to_path_base_dir: Option::arbitrary(g),
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

  #[tauri_macros::module_command_test(fs_read_file, "fs > readFile")]
  #[quickcheck_macros::quickcheck]
  fn read_file(path: SafePathBuf, options: Option<ReadFileOptions>) {
    let res = super::Cmd::read_file(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_write_file, "fs > writeFile")]
  #[quickcheck_macros::quickcheck]
  fn write_file(path: SafePathBuf, data: String, options: Option<WriteFileOptions>) {
    let res = super::Cmd::write_text_file(crate::test::mock_invoke_context(), path, data, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_read_dir, "fs > readDir")]
  #[quickcheck_macros::quickcheck]
  fn read_dir(path: SafePathBuf, options: Option<DirOperationOptions>) {
    let res = super::Cmd::read_dir(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_copy_file, "fs > copyFile")]
  #[quickcheck_macros::quickcheck]
  fn copy_file(from_path: SafePathBuf, to_path: SafePathBuf, options: Option<CopyFileOptions>) {
    let res = super::Cmd::copy_file(
      crate::test::mock_invoke_context(),
      from_path,
      to_path,
      options,
    );
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_create_dir, "fs > createDir")]
  #[quickcheck_macros::quickcheck]
  fn create_dir(path: SafePathBuf, options: Option<DirOperationOptions>) {
    let res = super::Cmd::create_dir(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_remove_dir, "fs > removeDir")]
  #[quickcheck_macros::quickcheck]
  fn remove_dir(path: SafePathBuf, options: Option<DirOperationOptions>) {
    let res = super::Cmd::remove_dir(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_remove_file, "fs > removeFile")]
  #[quickcheck_macros::quickcheck]
  fn remove_file(path: SafePathBuf, options: Option<FileOperationOptions>) {
    let res = super::Cmd::remove_file(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_rename_file, "fs > renameFile")]
  #[quickcheck_macros::quickcheck]
  fn rename_file(
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<FileOperationOptions>,
  ) {
    let res = super::Cmd::rename_file(
      crate::test::mock_invoke_context(),
      old_path,
      new_path,
      options,
    );
    crate::test_utils::assert_not_allowlist_error(res);
  }
}
