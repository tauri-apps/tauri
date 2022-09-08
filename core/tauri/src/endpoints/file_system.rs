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
use tauri_utils::atomic_counter::AtomicCounter;

use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  fmt::{Debug, Formatter},
  io::Read,
  sync::Mutex,
};
use std::{
  fs,
  fs::File,
  io::Write,
  path::{Component, Path},
  sync::Arc,
};

type Rid = u32;
type FsFileStore = Arc<Mutex<HashMap<Rid, File>>>;

static RID_COUNTER: AtomicCounter = AtomicCounter::new();
static FILES_STORE: Lazy<FsFileStore> = Lazy::new(Default::default);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateOptions {
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenOptions {
  read: Option<bool>,
  write: Option<bool>,
  append: Option<bool>,
  truncate: Option<bool>,
  create: Option<bool>,
  create_new: Option<bool>,
  #[allow(unused)]
  mode: Option<u32>,
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CopyFileOptions {
  from_path_base_dir: Option<BaseDirectory>,
  to_path_base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MkdirOptions {
  #[allow(unused)]
  mode: Option<u32>,
  recursive: Option<bool>,
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadDirOptions {
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReadFileOptions {
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoveOptions {
  recursive: Option<bool>,
  base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameOptions {
  new_path_base_dir: Option<BaseDirectory>,
  old_path_base_dir: Option<BaseDirectory>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteFileOptions {
  append: Option<bool>,
  create: Option<bool>,
  #[allow(unused)]
  mode: Option<u32>,
  base_dir: Option<BaseDirectory>,
}

/// The API descriptor.
#[command_enum]
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub(crate) enum Cmd {
  /// The create file API.
  #[cmd(fs_create, "fs > create")]
  Create {
    path: SafePathBuf,
    options: Option<CreateOptions>,
  },
  /// The open file API.
  #[cmd(fs_open, "fs > open")]
  Open {
    path: SafePathBuf,
    options: Option<OpenOptions>,
  },
  /// The close file API.
  #[cmd(fs_close, "fs > close")]
  Close { rid: Rid },
  /// The copy file API.
  #[cmd(fs_copy_file, "fs > copyFile")]
  #[serde(rename_all = "camelCase")]
  CopyFile {
    from_path: SafePathBuf,
    to_path: SafePathBuf,
    options: Option<CopyFileOptions>,
  },
  /// The create dir API.
  #[cmd(fs_mkdir, "fs > mkdir")]
  Mkdir {
    path: SafePathBuf,
    options: Option<MkdirOptions>,
  },
  /// The read dir API.
  #[cmd(fs_read_dir, "fs > readDir")]
  ReadDir {
    path: SafePathBuf,
    options: Option<ReadDirOptions>,
  },
  /// The read file API.
  #[cmd(fs_read_file, "fs > readFile")]
  Read { rid: Rid, len: u32 },
  #[cmd(fs_read_file, "fs > readFile")]
  ReadFile {
    path: SafePathBuf,
    options: Option<ReadFileOptions>,
  },
  #[cmd(fs_read_file, "fs > readFile")]
  ReadTextFile {
    path: SafePathBuf,
    options: Option<ReadFileOptions>,
  },
  /// The remove API.
  #[cmd(fs_remove, "fs > remove")]
  Remove {
    path: SafePathBuf,
    options: Option<RemoveOptions>,
  },
  /// The rename API.
  #[cmd(fs_rename, "fs > rename")]
  #[serde(rename_all = "camelCase")]
  Rename {
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<RenameOptions>,
  },
  /// The write file API.
  #[cmd(fs_write_file, "fs > writeFile")]
  Write { rid: Rid, data: Vec<u8> },
  #[cmd(fs_write_file, "fs > writeFile")]
  WriteFile {
    path: SafePathBuf,
    data: Vec<u8>,
    options: Option<WriteFileOptions>,
  },
  #[cmd(fs_write_file, "fs > writeFile")]
  WriteTextFile {
    path: SafePathBuf,
    data: String,
    options: Option<WriteFileOptions>,
  },
}

impl Cmd {
  #[module_command_handler(fs_create)]
  fn create<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<CreateOptions>,
  ) -> super::Result<Rid> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;

    let file = File::create(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(into_anyhow)?;

    let rid = RID_COUNTER.next();
    FILES_STORE.lock().unwrap().insert(rid, file);

    Ok(rid)
  }

  #[module_command_handler(fs_open)]
  fn open<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<OpenOptions>,
  ) -> super::Result<Rid> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;

    let mut opts = fs::OpenOptions::new();

    match options {
      None => {
        opts
          .read(true)
          .create(false)
          .write(false)
          .truncate(false)
          .append(false)
          .create_new(false);
      }
      Some(options) => {
        #[cfg(unix)]
        {
          use std::os::unix::fs::OpenOptionsExt;
          if let Some(mode) = options.mode {
            opts.mode(mode & 0o777);
          }
        }

        opts
          .read(options.read.unwrap_or(true))
          .create(options.create.unwrap_or(false))
          .write(options.write.unwrap_or(false))
          .truncate(options.truncate.unwrap_or(false))
          .append(options.append.unwrap_or(false))
          .create_new(options.create_new.unwrap_or(false));
      }
    }

    let file = opts
      .open(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(into_anyhow)?;

    let rid = RID_COUNTER.next();
    FILES_STORE.lock().unwrap().insert(rid, file);

    Ok(rid)
  }

  #[module_command_handler(fs_close)]
  fn close<R: Runtime>(_context: InvokeContext<R>, rid: Rid) -> super::Result<()> {
    FILES_STORE.lock().unwrap().remove(&rid);
    Ok(())
  }

  #[module_command_handler(fs_copy_file)]
  fn copy_file<R: Runtime>(
    context: InvokeContext<R>,
    from_path: SafePathBuf,
    to_path: SafePathBuf,
    options: Option<CopyFileOptions>,
  ) -> super::Result<()> {
    let resolved_from_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      file_url_to_safe_pathbuf(from_path)?,
      options.as_ref().and_then(|o| o.from_path_base_dir),
    )?;
    let resolved_to_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      file_url_to_safe_pathbuf(to_path)?,
      options.as_ref().and_then(|o| o.to_path_base_dir),
    )?;
    fs::copy(&resolved_from_path, &resolved_to_path).with_context(|| {
      format!(
        "fromPath: {}, toPath: {}",
        resolved_from_path.display(),
        resolved_to_path.display()
      )
    })?;
    Ok(())
  }

  #[module_command_handler(fs_mkdir)]
  fn mkdir<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<MkdirOptions>,
  ) -> super::Result<()> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;

    let mut builder = std::fs::DirBuilder::new();
    builder.recursive(options.as_ref().and_then(|o| o.recursive).unwrap_or(false));
    #[cfg(unix)]
    {
      use std::os::unix::fs::DirBuilderExt;
      let mode = options.as_ref().and_then(|o| o.mode).unwrap_or(0o777) & 0o777;
      builder.mode(mode);
    }

    builder
      .create(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
  }

  #[module_command_handler(fs_read_dir)]
  fn read_dir<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<ReadDirOptions>,
  ) -> super::Result<Vec<dir::DirEntry>> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;

    dir::read_dir(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(Into::into)
  }

  #[module_command_handler(fs_write_file)]
  fn read<R: Runtime>(
    _context: InvokeContext<R>,
    rid: Rid,
    len: u32,
  ) -> super::Result<(Vec<u8>, usize)> {
    if let Some(file) = FILES_STORE.lock().unwrap().get_mut(&rid) {
      let mut data = Vec::new();
      data.resize(len as usize, 0);
      let nread = file.read(&mut data).map_err(into_anyhow)?;
      Ok((data, nread))
    } else {
      Ok((Vec::new(), 0))
    }
  }

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

  #[module_command_handler(fs_remove)]
  fn remove<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<RemoveOptions>,
  ) -> super::Result<()> {
    let path = file_url_to_safe_pathbuf(path)?;

    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
    )?;

    let metadata = fs::symlink_metadata(&resolved_path)?;
    let file_type = metadata.file_type();

    // thank you deno devs, taken from: https://github.com/denoland/deno/blob/429759fe8b4207240709c240a8344d12a1e39566/runtime/ops/fs.rs#L728
    let res = if file_type.is_file() {
      fs::remove_file(&resolved_path)
    } else if options.as_ref().and_then(|o| o.recursive).unwrap_or(false) {
      fs::remove_dir_all(&resolved_path)
    } else if file_type.is_symlink() {
      #[cfg(unix)]
      {
        fs::remove_file(&resolved_path)
      }
      #[cfg(not(unix))]
      {
        use std::os::windows::fs::MetadataExt;
        const FILE_ATTRIBUTE_DIRECTORY: u32 = 0x00000010;
        if metadata.file_attributes() & FILE_ATTRIBUTE_DIRECTORY != 0 {
          fs::remove_dir(&resolved_path)
        } else {
          fs::remove_file(&resolved_path)
        }
      }
    } else if file_type.is_dir() {
      fs::remove_dir(&resolved_path)
    } else {
      // pipes, sockets, etc...
      std::fs::remove_file(&resolved_path)
    };

    res.with_context(|| format!("path: {}", resolved_path.display()))?;
    Ok(())
  }

  #[module_command_handler(fs_rename)]
  fn rename<R: Runtime>(
    context: InvokeContext<R>,
    old_path: SafePathBuf,
    new_path: SafePathBuf,
    options: Option<RenameOptions>,
  ) -> super::Result<()> {
    let resolved_old_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      file_url_to_safe_pathbuf(old_path)?,
      options.as_ref().and_then(|o| o.old_path_base_dir),
    )?;
    let resolved_new_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      file_url_to_safe_pathbuf(new_path)?,
      options.as_ref().and_then(|o| o.new_path_base_dir),
    )?;
    fs::rename(&resolved_old_path, &resolved_new_path)
      .with_context(|| {
        format!(
          "oldPath: {}, newPath: {}",
          resolved_old_path.display(),
          resolved_new_path.display()
        )
      })
      .map_err(Into::into)
  }

  #[module_command_handler(fs_write_file)]
  fn write<R: Runtime>(
    _context: InvokeContext<R>,
    rid: Rid,
    data: Vec<u8>,
  ) -> super::Result<usize> {
    if let Some(file) = FILES_STORE.lock().unwrap().get_mut(&rid) {
      file.write(&data).map_err(Into::into)
    } else {
      Ok(0)
    }
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

  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    if let Some(Some(mode)) = options.map(|o| o.mode) {
      opts.mode(mode & 0o777);
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
    BaseDirectory, CopyFileOptions, CreateOptions, MkdirOptions, OpenOptions, ReadDirOptions,
    ReadFileOptions, RemoveOptions, RenameOptions, Rid, SafePathBuf, WriteFileOptions,
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

  impl Arbitrary for WriteFileOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        append: Option::arbitrary(g),
        create: Option::arbitrary(g),
        mode: Option::arbitrary(g),
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

  impl Arbitrary for ReadDirOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for MkdirOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        mode: Option::arbitrary(g),
        recursive: Option::arbitrary(g),
        base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for RemoveOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        recursive: Option::arbitrary(g),
        base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for RenameOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        old_path_base_dir: Option::arbitrary(g),
        new_path_base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for CreateOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        base_dir: Option::arbitrary(g),
      }
    }
  }

  impl Arbitrary for OpenOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        read: Option::arbitrary(g),
        write: Option::arbitrary(g),
        append: Option::arbitrary(g),
        truncate: Option::arbitrary(g),
        create: Option::arbitrary(g),
        create_new: Option::arbitrary(g),
        mode: Option::arbitrary(g),
        base_dir: Option::arbitrary(g),
      }
    }
  }

  #[tauri_macros::module_command_test(fs_create, "fs > create")]
  #[quickcheck_macros::quickcheck]
  fn create(path: SafePathBuf, options: Option<CreateOptions>) {
    let res = super::Cmd::create(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }
  #[tauri_macros::module_command_test(fs_open, "fs > open")]
  #[quickcheck_macros::quickcheck]
  fn open(path: SafePathBuf, options: Option<OpenOptions>) {
    let res = super::Cmd::open(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_close, "fs > close")]
  #[quickcheck_macros::quickcheck]
  fn close(rid: Rid) {
    let res = super::Cmd::close(crate::test::mock_invoke_context(), rid);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_read_file, "fs > readFile")]
  #[quickcheck_macros::quickcheck]
  fn read_file(path: SafePathBuf, options: Option<ReadFileOptions>) {
    let res = super::Cmd::read_file(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_write_file, "fs > readFile")]
  #[quickcheck_macros::quickcheck]
  fn read(rid: Rid, len: u32) {
    let res = super::Cmd::read(crate::test::mock_invoke_context(), rid, len);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_write_file, "fs > writeFile")]
  #[quickcheck_macros::quickcheck]
  fn write(rid: Rid, data: Vec<u8>) {
    let res = super::Cmd::write(crate::test::mock_invoke_context(), rid, data);
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
  fn read_dir(path: SafePathBuf, options: Option<ReadDirOptions>) {
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

  #[tauri_macros::module_command_test(fs_mkdir, "fs > mkdir")]
  #[quickcheck_macros::quickcheck]
  fn mkdir(path: SafePathBuf, options: Option<MkdirOptions>) {
    let res = super::Cmd::mkdir(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_remove, "fs > remove")]
  #[quickcheck_macros::quickcheck]
  fn remove(path: SafePathBuf, options: Option<RemoveOptions>) {
    let res = super::Cmd::remove(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_rename, "fs > rename")]
  #[quickcheck_macros::quickcheck]
  fn rename(old_path: SafePathBuf, new_path: SafePathBuf, options: Option<RenameOptions>) {
    let res = super::Cmd::rename(
      crate::test::mock_invoke_context(),
      old_path,
      new_path,
      options,
    );
    crate::test_utils::assert_not_allowlist_error(res);
  }
}
