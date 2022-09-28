// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  borrow::Cow,
  fs,
  io::{self, Cursor, Read, Seek},
  path::{self, Path, PathBuf},
};

/// The archive reader.
#[derive(Debug)]
pub enum ArchiveReader<R: Read + Seek> {
  /// A plain reader.
  Plain(R),
  /// A GZ- compressed reader (decoder).
  GzCompressed(flate2::read::GzDecoder<R>),
}

impl<R: Read + Seek> Read for ArchiveReader<R> {
  fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
    match self {
      Self::Plain(r) => r.read(buf),
      Self::GzCompressed(decoder) => decoder.read(buf),
    }
  }
}

impl<R: Read + Seek> ArchiveReader<R> {
  #[allow(dead_code)]
  fn get_mut(&mut self) -> &mut R {
    match self {
      Self::Plain(r) => r,
      Self::GzCompressed(decoder) => decoder.get_mut(),
    }
  }
}

/// The supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum ArchiveFormat {
  /// Tar archive.
  Tar(Option<Compression>),
  /// Zip archive.
  Zip,
}

/// The supported compression types.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Compression {
  /// Gz compression (e.g. `.tar.gz` archives)
  Gz,
}

/// The zip entry.
pub struct ZipEntry {
  path: PathBuf,
  is_dir: bool,
  file_contents: Vec<u8>,
}

/// A read-only view into an entry of an archive.
#[non_exhaustive]
pub enum Entry<'a, R: Read> {
  /// An entry of a tar archive.
  #[non_exhaustive]
  Tar(Box<tar::Entry<'a, R>>),
  /// An entry of a zip archive.
  #[non_exhaustive]
  Zip(ZipEntry),
}

impl<'a, R: Read> Entry<'a, R> {
  /// The entry path.
  pub fn path(&self) -> crate::api::Result<Cow<'_, Path>> {
    match self {
      Self::Tar(e) => e.path().map_err(Into::into),
      Self::Zip(e) => Ok(Cow::Borrowed(&e.path)),
    }
  }

  /// Extract this entry into `into_path`.
  /// If it's a directory, the target will be created, if it's a file, it'll be extracted at this location.
  /// Note: You need to include the complete path, with file name and extension.
  pub fn extract(self, into_path: &path::Path) -> crate::api::Result<()> {
    match self {
      Self::Tar(mut entry) => {
        // determine if it's a file or a directory
        if entry.header().entry_type() == tar::EntryType::Directory {
          // this is a directory, lets create it
          match fs::create_dir_all(into_path) {
            Ok(_) => (),
            Err(e) => {
              if e.kind() != io::ErrorKind::AlreadyExists {
                return Err(e.into());
              }
            }
          }
        } else {
          let mut out_file = fs::File::create(into_path)?;
          io::copy(&mut entry, &mut out_file)?;

          // make sure we set permissions
          if let Ok(mode) = entry.header().mode() {
            set_perms(into_path, Some(&mut out_file), mode, true)?;
          }
        }
      }
      Self::Zip(entry) => {
        if entry.is_dir {
          // this is a directory, lets create it
          match fs::create_dir_all(into_path) {
            Ok(_) => (),
            Err(e) => {
              if e.kind() != io::ErrorKind::AlreadyExists {
                return Err(e.into());
              }
            }
          }
        } else {
          let mut out_file = fs::File::create(into_path)?;
          io::copy(&mut Cursor::new(entry.file_contents), &mut out_file)?;
        }
      }
    }

    Ok(())
  }
}

/// The extract manager to retrieve files from archives.
pub struct Extract<'a, R: Read + Seek> {
  reader: ArchiveReader<R>,
  archive_format: ArchiveFormat,
  tar_archive: Option<tar::Archive<&'a mut ArchiveReader<R>>>,
}

impl<'a, R: std::fmt::Debug + Read + Seek> std::fmt::Debug for Extract<'a, R> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("Extract")
      .field("reader", &self.reader)
      .field("archive_format", &self.archive_format)
      .finish()
  }
}

impl<'a, R: Read + Seek> Extract<'a, R> {
  /// Create archive from reader.
  pub fn from_cursor(mut reader: R, archive_format: ArchiveFormat) -> Extract<'a, R> {
    if reader.seek(io::SeekFrom::Start(0)).is_err() {
      #[cfg(debug_assertions)]
      eprintln!("Could not seek to start of the file");
    }
    let compression = if let ArchiveFormat::Tar(compression) = archive_format {
      compression
    } else {
      None
    };
    Extract {
      reader: match compression {
        Some(Compression::Gz) => ArchiveReader::GzCompressed(flate2::read::GzDecoder::new(reader)),
        _ => ArchiveReader::Plain(reader),
      },
      archive_format,
      tar_archive: None,
    }
  }

  /// Reads the archive content.
  pub fn with_files<
    E: Into<crate::api::Error>,
    F: FnMut(Entry<'_, &mut ArchiveReader<R>>) -> std::result::Result<bool, E>,
  >(
    &'a mut self,
    mut f: F,
  ) -> crate::api::Result<()> {
    match self.archive_format {
      ArchiveFormat::Tar(_) => {
        let archive = tar::Archive::new(&mut self.reader);
        self.tar_archive.replace(archive);
        for entry in self.tar_archive.as_mut().unwrap().entries()? {
          let entry = entry?;
          if entry.path().is_ok() {
            let stop = f(Entry::Tar(Box::new(entry))).map_err(Into::into)?;
            if stop {
              break;
            }
          }
        }
      }

      ArchiveFormat::Zip => {
        #[cfg(feature = "fs-extract-api")]
        {
          let mut archive = zip::ZipArchive::new(self.reader.get_mut())?;
          let file_names = archive
            .file_names()
            .map(|f| f.to_string())
            .collect::<Vec<String>>();
          for path in file_names {
            let mut zip_file = archive.by_name(&path)?;
            let is_dir = zip_file.is_dir();
            let mut file_contents = Vec::new();
            zip_file.read_to_end(&mut file_contents)?;
            let stop = f(Entry::Zip(ZipEntry {
              path: path.into(),
              is_dir,
              file_contents,
            }))
            .map_err(Into::into)?;
            if stop {
              break;
            }
          }
        }
      }
    }

    Ok(())
  }

  /// Extract an entire source archive into a specified path. If the source is a single compressed
  /// file and not an archive, it will be extracted into a file with the same name inside of
  /// `into_dir`.
  pub fn extract_into(&mut self, into_dir: &path::Path) -> crate::api::Result<()> {
    match self.archive_format {
      ArchiveFormat::Tar(_) => {
        let mut archive = tar::Archive::new(&mut self.reader);
        archive.unpack(into_dir)?;
      }

      ArchiveFormat::Zip => {
        #[cfg(feature = "fs-extract-api")]
        {
          let mut archive = zip::ZipArchive::new(self.reader.get_mut())?;
          for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            // Decode the file name from raw bytes instead of using file.name() directly.
            // file.name() uses String::from_utf8_lossy() which may return messy characters
            // such as: τê▒Σ║ñµÿô.app/, that does not work as expected.
            // Here we require the file name must be a valid UTF-8.
            let file_name = String::from_utf8(file.name_raw().to_vec())?;
            let out_path = into_dir.join(&file_name);
            if file.is_dir() {
              fs::create_dir_all(&out_path)?;
            } else {
              if let Some(out_path_parent) = out_path.parent() {
                fs::create_dir_all(&out_path_parent)?;
              }
              let mut out_file = fs::File::create(&out_path)?;
              io::copy(&mut file, &mut out_file)?;
            }
            // Get and Set permissions
            #[cfg(unix)]
            {
              use std::os::unix::fs::PermissionsExt;
              if let Some(mode) = file.unix_mode() {
                fs::set_permissions(&out_path, fs::Permissions::from_mode(mode))?;
              }
            }
          }
        }
      }
    }
    Ok(())
  }
}

fn set_perms(
  dst: &Path,
  f: Option<&mut std::fs::File>,
  mode: u32,
  preserve: bool,
) -> crate::api::Result<()> {
  _set_perms(dst, f, mode, preserve).map_err(|_| {
    crate::api::Error::Extract(format!(
      "failed to set permissions to {:o} \
               for `{}`",
      mode,
      dst.display()
    ))
  })
}

#[cfg(unix)]
fn _set_perms(
  dst: &Path,
  f: Option<&mut std::fs::File>,
  mode: u32,
  preserve: bool,
) -> io::Result<()> {
  use std::os::unix::prelude::*;

  let mode = if preserve { mode } else { mode & 0o777 };
  let perm = fs::Permissions::from_mode(mode as _);
  match f {
    Some(f) => f.set_permissions(perm),
    None => fs::set_permissions(dst, perm),
  }
}

#[cfg(windows)]
fn _set_perms(
  dst: &Path,
  f: Option<&mut std::fs::File>,
  mode: u32,
  _preserve: bool,
) -> io::Result<()> {
  if mode & 0o200 == 0o200 {
    return Ok(());
  }
  match f {
    Some(f) => {
      let mut perm = f.metadata()?.permissions();
      perm.set_readonly(true);
      f.set_permissions(perm)
    }
    None => {
      let mut perm = fs::metadata(dst)?.permissions();
      perm.set_readonly(true);
      fs::set_permissions(dst, perm)
    }
  }
}
