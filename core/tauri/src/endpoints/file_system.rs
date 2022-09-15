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
use anyhow::Context;
use serde::{
  de::{Deserializer, Error as DeError},
  Deserialize, Serialize,
};
use serde_repr::{Deserialize_repr, Serialize_repr};
use tauri_macros::{command_enum, module_command_handler, CommandModule};
use tauri_utils::atomic_counter::AtomicCounter;

use once_cell::sync::Lazy;
use std::{
  collections::HashMap,
  fmt::{Debug, Formatter},
  io::{self, Read},
  sync::Mutex,
  time::{SystemTime, UNIX_EPOCH},
};
use std::{
  fs,
  fs::File,
  io::Write,
  path::{Component, Path},
  sync::Arc,
};

/// Resource id
type Rid = u32;
type FsFileStore = Arc<Mutex<HashMap<Rid, File>>>;

static RID_COUNTER: AtomicCounter = AtomicCounter::new();
static FILES_STORE: Lazy<FsFileStore> = Lazy::new(Default::default);

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GenericOptions {
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

#[derive(Serialize_repr, Deserialize_repr, Clone, Copy, Debug)]
#[repr(u16)]
pub enum SeekMode {
  Start = 0,
  Current = 1,
  End = 2,
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
    options: Option<GenericOptions>,
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
    options: Option<GenericOptions>,
  },
  /// The read file API.
  #[cmd(fs_read_file, "fs > readFile")]
  Read { rid: Rid, len: u32 },
  #[cmd(fs_read_file, "fs > readFile")]
  ReadFile {
    path: SafePathBuf,
    options: Option<GenericOptions>,
  },
  #[cmd(fs_read_file, "fs > readFile")]
  ReadTextFile {
    path: SafePathBuf,
    options: Option<GenericOptions>,
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
  /// The seek file API
  #[cmd(fs_seek_file, "fs > writeFile or fs > readFile")]
  Seek {
    rid: Rid,
    offset: i64,
    whence: SeekMode,
  },
  /// The stat file API
  #[cmd(fs_read_file, "fs > readFile")]
  Stat {
    path: SafePathBuf,
    options: Option<GenericOptions>,
  },
  #[cmd(fs_read_file, "fs > readFile")]
  Lstat {
    path: SafePathBuf,
    options: Option<GenericOptions>,
  },
  #[cmd(fs_read_file, "fs > readFile")]
  Fstat { rid: Rid },
  /// The truncate file API
  #[cmd(fs_write_file, "fs > writeFile")]
  Truncate {
    path: SafePathBuf,
    len: Option<u64>,
    options: Option<GenericOptions>,
  },
  #[cmd(fs_write_file, "fs > writeFile")]
  Ftruncate { rid: Rid, len: Option<u64> },
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
    options: Option<GenericOptions>,
  ) -> super::Result<Rid> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
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
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
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
      from_path,
      options.as_ref().and_then(|o| o.from_path_base_dir),
      true,
    )?;
    let resolved_to_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      to_path,
      options.as_ref().and_then(|o| o.to_path_base_dir),
      true,
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
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
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
    options: Option<GenericOptions>,
  ) -> super::Result<Vec<dir::DirEntry>> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
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
    let mut data = Vec::new();
    data.resize(len as usize, 0);
    let mut store = FILES_STORE.lock().unwrap();
    let file = store
      .get_mut(&rid)
      .with_context(|| format!("Incorrect file rid: {}", rid))?;
    let nread = file.read(&mut data).map_err(into_anyhow)?;
    Ok((data, nread))
  }

  #[module_command_handler(fs_read_file)]
  fn read_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<GenericOptions>,
  ) -> super::Result<Vec<u8>> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
    )?;
    file::read_binary(&resolved_path)
      .with_context(|| format!("path: {}", resolved_path.display()))
      .map_err(Into::into)
  }

  #[module_command_handler(fs_read_file)]
  fn read_text_file<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<GenericOptions>,
  ) -> super::Result<String> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
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
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
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
      old_path,
      options.as_ref().and_then(|o| o.old_path_base_dir),
      true,
    )?;
    let resolved_new_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      new_path,
      options.as_ref().and_then(|o| o.new_path_base_dir),
      true,
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

  #[module_command_handler(fs_seek_file)]
  fn seek<R: Runtime>(
    _context: InvokeContext<R>,
    rid: Rid,
    offset: i64,
    whence: SeekMode,
  ) -> super::Result<u64> {
    use std::io::{Seek, SeekFrom};
    let mut store = FILES_STORE.lock().unwrap();
    let file = store
      .get_mut(&rid)
      .with_context(|| format!("Incorrect file rid: {}", rid))?;
    file
      .seek(match whence {
        SeekMode::Start => SeekFrom::Start(offset as u64),
        SeekMode::Current => SeekFrom::Current(offset),
        SeekMode::End => SeekFrom::End(offset),
      })
      .map_err(into_anyhow)
  }

  #[module_command_handler(fs_read_file)]
  fn stat<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<GenericOptions>,
  ) -> super::Result<FileInfo> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
    )?;

    let metadata = fs::metadata(&resolved_path)?;
    Ok(get_stat(metadata))
  }

  #[module_command_handler(fs_read_file)]
  fn lstat<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    options: Option<GenericOptions>,
  ) -> super::Result<FileInfo> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      false,
    )?;
    dbg!(&resolved_path);
    let metadata = fs::symlink_metadata(&resolved_path)?;
    Ok(get_stat(metadata))
  }

  #[module_command_handler(fs_read_file)]
  fn fstat<R: Runtime>(_context: InvokeContext<R>, rid: Rid) -> super::Result<FileInfo> {
    let mut store = FILES_STORE.lock().unwrap();
    let file = store
      .get_mut(&rid)
      .with_context(|| format!("Incorrect file rid: {}", rid))?;
    let metadata = file.metadata()?;
    Ok(get_stat(metadata))
  }

  #[module_command_handler(fs_write_file)]
  fn truncate<R: Runtime>(
    context: InvokeContext<R>,
    path: SafePathBuf,
    len: Option<u64>,
    options: Option<GenericOptions>,
  ) -> super::Result<()> {
    let resolved_path = resolve_path(
      &context.config,
      &context.package_info,
      &context.window,
      path,
      options.as_ref().and_then(|o| o.base_dir),
      true,
    )?;

    let f = std::fs::OpenOptions::new()
      .write(true)
      .open(&resolved_path)?;
    f.set_len(len.unwrap_or(0)).map_err(into_anyhow)
  }
  #[module_command_handler(fs_write_file)]
  fn ftruncate<R: Runtime>(
    _context: InvokeContext<R>,
    rid: Rid,
    len: Option<u64>,
  ) -> super::Result<()> {
    let mut store = FILES_STORE.lock().unwrap();
    let file = store
      .get_mut(&rid)
      .with_context(|| format!("Incorrect file rid: {}", rid))?;
    file.set_len(len.unwrap_or(0)).map_err(into_anyhow)
  }

  #[module_command_handler(fs_write_file)]
  fn write<R: Runtime>(
    _context: InvokeContext<R>,
    rid: Rid,
    data: Vec<u8>,
  ) -> super::Result<usize> {
    let mut store = FILES_STORE.lock().unwrap();
    let file = store
      .get_mut(&rid)
      .with_context(|| format!("Incorrect file rid: {}", rid))?;
    file.write(&data).map_err(Into::into)
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

fn write_file<R: Runtime>(
  context: InvokeContext<R>,
  path: SafePathBuf,
  data: &[u8],
  options: Option<WriteFileOptions>,
) -> super::Result<()> {
  let resolved_path = resolve_path(
    &context.config,
    &context.package_info,
    &context.window,
    path,
    options.as_ref().and_then(|o| o.base_dir),
    true,
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

#[allow(dead_code)]
pub(crate) fn resolve_path<R: Runtime>(
  config: &Config,
  package_info: &PackageInfo,
  window: &Window<R>,
  path: SafePathBuf,
  dir: Option<BaseDirectory>,
  follow_symlink: bool,
) -> super::Result<SafePathBuf> {
  let path = file_url_to_safe_pathbuf(path)?;
  let env = window.state::<Env>().inner();
  match crate::api::path::resolve_path(config, package_info, env, &path, dir) {
    Ok(path) => {
      let allowed = if follow_symlink {
        window.state::<Scopes>().fs.is_allowed(&path)
      } else {
        window.state::<Scopes>().fs.is_symlink_allowed(&path)
      };
      if allowed {
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

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L913
fn to_msec(maybe_time: Result<SystemTime, io::Error>) -> Option<u64> {
  match maybe_time {
    Ok(time) => {
      let msec = time
        .duration_since(UNIX_EPOCH)
        .map(|t| t.as_millis() as u64)
        .unwrap_or_else(|err| err.duration().as_millis() as u64);
      Some(msec)
    }
    Err(_) => None,
  }
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L926
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileInfo {
  is_file: bool,
  is_directory: bool,
  is_symlink: bool,
  size: u64,
  // In milliseconds, like JavaScript. Available on both Unix or Windows.
  mtime: Option<u64>,
  atime: Option<u64>,
  birthtime: Option<u64>,
  // Following are only valid under Unix.
  dev: u64,
  ino: u64,
  mode: u32,
  nlink: u64,
  uid: u32,
  gid: u32,
  rdev: u64,
  blksize: u64,
  blocks: u64,
}

// taken from deno source code: https://github.com/denoland/deno/blob/ffffa2f7c44bd26aec5ae1957e0534487d099f48/runtime/ops/fs.rs#L950
#[inline(always)]
fn get_stat(metadata: std::fs::Metadata) -> FileInfo {
  // Unix stat member (number types only). 0 if not on unix.
  macro_rules! usm {
    ($member:ident) => {{
      #[cfg(unix)]
      {
        metadata.$member()
      }
      #[cfg(not(unix))]
      {
        0
      }
    }};
  }

  #[cfg(unix)]
  use std::os::unix::fs::MetadataExt;
  FileInfo {
    is_file: metadata.is_file(),
    is_directory: metadata.is_dir(),
    is_symlink: metadata.file_type().is_symlink(),
    size: metadata.len(),
    // In milliseconds, like JavaScript. Available on both Unix or Windows.
    mtime: to_msec(metadata.modified()),
    atime: to_msec(metadata.accessed()),
    birthtime: to_msec(metadata.created()),
    // Following are only valid under Unix.
    dev: usm!(dev),
    ino: usm!(ino),
    mode: usm!(mode),
    nlink: usm!(nlink),
    uid: usm!(uid),
    gid: usm!(gid),
    rdev: usm!(rdev),
    blksize: usm!(blksize),
    blocks: usm!(blocks),
  }
}

#[cfg(test)]
mod tests {
  use super::{
    BaseDirectory, CopyFileOptions, GenericOptions, MkdirOptions, OpenOptions, RemoveOptions,
    RenameOptions, Rid, SafePathBuf, SeekMode, WriteFileOptions,
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

  impl Arbitrary for SeekMode {
    fn arbitrary(g: &mut Gen) -> Self {
      if bool::arbitrary(g) {
        SeekMode::Current
      } else {
        SeekMode::Start
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

  impl Arbitrary for GenericOptions {
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
  fn create(path: SafePathBuf, options: Option<GenericOptions>) {
    let res = super::Cmd::create(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }
  #[tauri_macros::module_command_test(fs_open, "fs > open")]
  #[quickcheck_macros::quickcheck]
  fn open(path: SafePathBuf, options: Option<OpenOptions>) {
    let res = super::Cmd::open(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_read_file, "fs > readFile")]
  #[quickcheck_macros::quickcheck]
  fn read_file(path: SafePathBuf, options: Option<GenericOptions>) {
    let res = super::Cmd::read_file(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_read_file, "fs > readFile")]
  #[quickcheck_macros::quickcheck]
  fn stat(path: SafePathBuf, options: Option<GenericOptions>) {
    let res = super::Cmd::stat(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_read_file, "fs > readFile")]
  #[quickcheck_macros::quickcheck]
  fn lstat(path: SafePathBuf, options: Option<GenericOptions>) {
    let res = super::Cmd::lstat(crate::test::mock_invoke_context(), path, options);
    crate::test_utils::assert_not_allowlist_error(res);
  }

  #[tauri_macros::module_command_test(fs_write_file, "fs > writeFile")]
  #[quickcheck_macros::quickcheck]
  fn truncate(path: SafePathBuf, len: Option<u64>, options: Option<GenericOptions>) {
    let res = super::Cmd::truncate(crate::test::mock_invoke_context(), path, len, options);
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
  fn read_dir(path: SafePathBuf, options: Option<GenericOptions>) {
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
