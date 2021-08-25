// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use either::{self, Either};
use std::{
  fs,
  io::{self, Read, Seek},
  path::{self, Path, PathBuf},
};
use tar::EntryType;

/// The supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum ArchiveFormat {
  /// Tar archive.
  Tar(Option<Compression>),
  /// Plain archive.
  Plain(Option<Compression>),
  /// Zip archive.
  Zip,
}

/// The supported compression types.
#[derive(Debug, Clone, Copy, PartialEq)]
#[non_exhaustive]
pub enum Compression {
  /// Gz compression (e.g. `.tar.gz` archives)
  Gz,
}

/// The extract manager to retrieve files from archives.
#[derive(Debug)]
pub struct Extract<R> {
  reader: R,
  file_name: String,
  archive_format: ArchiveFormat,
}

impl<R: Read + Seek> Extract<R> {
  /// Create archive from reader
  pub fn from_cursor(mut reader: R, archive_format: ArchiveFormat) -> Extract<R> {
    if reader.seek(io::SeekFrom::Start(0)).is_err() {
      println!("Could not seek to start of the file");
    }
    Extract {
      reader,
      file_name: "archive".to_string(),
      archive_format,
    }
  }

  /// Get the files Path in the archive buffer.
  pub fn files(&mut self) -> crate::api::Result<Vec<PathBuf>> {
    let reader = &mut self.reader;
    let mut all_files = Vec::new();
    if reader.seek(io::SeekFrom::Start(0)).is_err() {
      println!("Could not seek to start of the file");
    }
    match self.archive_format {
      ArchiveFormat::Plain(compression) | ArchiveFormat::Tar(compression) => {
        let reader = Self::get_archive_reader(reader, compression);
        match self.archive_format {
          ArchiveFormat::Tar(_) => {
            let mut archive = tar::Archive::new(reader);
            for entry in archive.entries()?.flatten() {
              if let Ok(path) = entry.path() {
                all_files.push(path.to_path_buf());
              }
            }
          }
          _ => unreachable!(),
        };
      }

      ArchiveFormat::Zip => {
        let archive = zip::ZipArchive::new(reader)?;
        for entry in archive.file_names() {
          all_files.push(PathBuf::from(entry));
        }
      }
    }

    Ok(all_files)
  }

  // Get the reader based on the compression type.
  fn get_archive_reader(
    source: &mut R,
    compression: Option<Compression>,
  ) -> Either<&mut R, flate2::read::GzDecoder<&mut R>> {
    if source.seek(io::SeekFrom::Start(0)).is_err() {
      println!("Could not seek to start of the file");
    }
    match compression {
      Some(Compression::Gz) => Either::Right(flate2::read::GzDecoder::new(source)),
      None => Either::Left(source),
    }
  }

  /// Extract an entire source archive into a specified path. If the source is a single compressed
  /// file and not an archive, it will be extracted into a file with the same name inside of
  /// `into_dir`.
  pub fn extract_into(&mut self, into_dir: &path::Path) -> crate::api::Result<()> {
    let reader = &mut self.reader;
    if reader.seek(io::SeekFrom::Start(0)).is_err() {
      println!("Could not seek to start of the file");
    }
    match self.archive_format {
      ArchiveFormat::Plain(compression) | ArchiveFormat::Tar(compression) => {
        let mut reader = Self::get_archive_reader(reader, compression);
        match self.archive_format {
          ArchiveFormat::Plain(_) => {
            match fs::create_dir_all(into_dir) {
              Ok(_) => (),
              Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                  return Err(e.into());
                }
              }
            }

            let mut out_path = into_dir.join(&self.file_name);
            out_path.set_extension("");
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut reader, &mut out_file)?;
          }
          ArchiveFormat::Tar(_) => {
            let mut archive = tar::Archive::new(reader);
            archive.unpack(into_dir)?;
          }
          _ => unreachable!(),
        };
      }

      ArchiveFormat::Zip => {
        let mut archive = zip::ZipArchive::new(reader)?;
        for i in 0..archive.len() {
          let mut file = archive.by_index(i)?;
          let path = into_dir.join(file.name());
          let mut output = fs::File::create(path)?;
          io::copy(&mut file, &mut output)?;
        }
      }
    }
    Ok(())
  }

  /// Extract a single file from a source and save to a file of the same name in `into_dir`.
  /// If the source is a single compressed file, it will be saved with the name `file_to_extract`
  /// in the specified `into_dir`.
  pub fn extract_file<T: AsRef<path::Path>>(
    &mut self,
    into_path: &path::Path,
    file_to_extract: T,
  ) -> crate::api::Result<()> {
    let file_to_extract = file_to_extract.as_ref();
    let reader = &mut self.reader;

    match self.archive_format {
      ArchiveFormat::Plain(compression) | ArchiveFormat::Tar(compression) => {
        let mut reader = Self::get_archive_reader(reader, compression);
        match self.archive_format {
          ArchiveFormat::Plain(_) => {
            match fs::create_dir_all(into_path) {
              Ok(_) => (),
              Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                  return Err(e.into());
                }
              }
            }
            let out_path = into_path.join(&self.file_name);
            let mut out_file = fs::File::create(&out_path)?;
            io::copy(&mut reader, &mut out_file)?;
          }
          ArchiveFormat::Tar(_) => {
            let mut archive = tar::Archive::new(reader);
            let mut entry = archive
              .entries()?
              .filter_map(|e| e.ok())
              .find(|e| e.path().ok().filter(|p| p == file_to_extract).is_some())
              .ok_or_else(|| {
                crate::api::Error::Extract(format!(
                  "Could not find the required path in the archive: {:?}",
                  file_to_extract
                ))
              })?;

            if entry.header().entry_type() == EntryType::Directory {
              // this is a directory, lets create it
              fs::create_dir_all(into_path)?;
            } else {
              let mut file = &mut fs::File::create(into_path)?;
              io::copy(&mut entry, file)?;
              if let Ok(mode) = entry.header().mode() {
                set_perms(into_path, Some(&mut file), mode, false)?;
              }
            }
          }
          _ => {
            panic!("Unreasonable code");
          }
        };
      }
      ArchiveFormat::Zip => {
        let mut archive = zip::ZipArchive::new(reader)?;
        let mut file = archive.by_name(
          file_to_extract
            .to_str()
            .expect("Could not convert file to str"),
        )?;
        let mut output = fs::File::create(into_path.join(file.name()))?;
        io::copy(&mut file, &mut output)?;
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
