// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use either::{self, Either};

use std::{fs, io, path};

/// The supported archive formats.
#[derive(Debug, Clone, Copy, PartialEq)]
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
pub enum Compression {
  /// Gz compression (e.g. `.tar.gz` archives)
  Gz,
}

/// The extract manager.
#[derive(Debug)]
pub struct Extract<'a> {
  source: &'a path::Path,
  archive_format: Option<ArchiveFormat>,
}

fn detect_archive_type(path: &path::Path) -> ArchiveFormat {
  match path.extension() {
    Some(extension) if extension == std::ffi::OsStr::new("zip") => ArchiveFormat::Zip,
    Some(extension) if extension == std::ffi::OsStr::new("tar") => ArchiveFormat::Tar(None),
    Some(extension) if extension == std::ffi::OsStr::new("gz") => match path
      .file_stem()
      .map(|e| path::Path::new(e))
      .and_then(|f| f.extension())
    {
      Some(extension) if extension == std::ffi::OsStr::new("tar") => {
        ArchiveFormat::Tar(Some(Compression::Gz))
      }
      _ => ArchiveFormat::Plain(Some(Compression::Gz)),
    },
    _ => ArchiveFormat::Plain(None),
  }
}

impl<'a> Extract<'a> {
  /// Create an `Extractor from a source path
  pub fn from_source(source: &'a path::Path) -> Extract<'a> {
    Self {
      source,
      archive_format: None,
    }
  }

  /// Specify an archive format of the source being extracted. If not specified, the
  /// archive format will determined from the file extension.
  pub fn archive_format(&mut self, format: ArchiveFormat) -> &mut Self {
    self.archive_format = Some(format);
    self
  }

  fn get_archive_reader(
    source: fs::File,
    compression: Option<Compression>,
  ) -> Either<fs::File, flate2::read::GzDecoder<fs::File>> {
    match compression {
      Some(Compression::Gz) => Either::Right(flate2::read::GzDecoder::new(source)),
      None => Either::Left(source),
    }
  }

  /// Extract an entire source archive into a specified path. If the source is a single compressed
  /// file and not an archive, it will be extracted into a file with the same name inside of
  /// `into_dir`.
  pub fn extract_into(&self, into_dir: &path::Path) -> crate::api::Result<()> {
    let source = fs::File::open(self.source)?;
    let archive = self
      .archive_format
      .unwrap_or_else(|| detect_archive_type(&self.source));

    match archive {
      ArchiveFormat::Plain(compression) | ArchiveFormat::Tar(compression) => {
        let mut reader = Self::get_archive_reader(source, compression);

        match archive {
          ArchiveFormat::Plain(_) => {
            match fs::create_dir_all(into_dir) {
              Ok(_) => (),
              Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                  return Err(e.into());
                }
              }
            }
            let file_name = self.source.file_name().ok_or_else(|| {
              crate::api::Error::Extract("Extractor source has no file-name".into())
            })?;
            let mut out_path = into_dir.join(file_name);
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
        let mut archive = zip::ZipArchive::new(source)?;
        for i in 0..archive.len() {
          let mut file = archive.by_index(i)?;
          let path = into_dir.join(file.name());
          let mut output = fs::File::create(path)?;
          io::copy(&mut file, &mut output)?;
        }
      }
    };
    Ok(())
  }

  /// Extract a single file from a source and save to a file of the same name in `into_dir`.
  /// If the source is a single compressed file, it will be saved with the name `file_to_extract`
  /// in the specified `into_dir`.
  pub fn extract_file<T: AsRef<path::Path>>(
    &self,
    into_dir: &path::Path,
    file_to_extract: T,
  ) -> crate::api::Result<()> {
    let file_to_extract = file_to_extract.as_ref();
    let source = fs::File::open(self.source)?;
    let archive = self
      .archive_format
      .unwrap_or_else(|| detect_archive_type(&self.source));

    match archive {
      ArchiveFormat::Plain(compression) | ArchiveFormat::Tar(compression) => {
        let mut reader = Self::get_archive_reader(source, compression);

        match archive {
          ArchiveFormat::Plain(_) => {
            match fs::create_dir_all(into_dir) {
              Ok(_) => (),
              Err(e) => {
                if e.kind() != io::ErrorKind::AlreadyExists {
                  return Err(e.into());
                }
              }
            }
            let file_name = file_to_extract.file_name().ok_or_else(|| {
              crate::api::Error::Extract("Extractor source has no file-name".into())
            })?;
            let out_path = into_dir.join(file_name);
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
            entry.unpack_in(into_dir)?;
          }
          _ => {
            panic!("Unreasonable code");
          }
        };
      }
      ArchiveFormat::Zip => {
        let mut archive = zip::ZipArchive::new(source)?;
        let mut file = archive.by_name(
          file_to_extract
            .to_str()
            .expect("Could not convert file to str"),
        )?;
        let mut output = fs::File::create(into_dir.join(file.name()))?;
        io::copy(&mut file, &mut output)?;
      }
    };
    Ok(())
  }
}
