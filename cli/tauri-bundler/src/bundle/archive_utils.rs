use super::common;
use libflate::gzip;
use std::fs::{self};
use std::io::Write;
use std::path::{Path, PathBuf};
use tar;
use walkdir::WalkDir;

#[cfg(target_os = "windows")]
use std::fs::File;
#[cfg(target_os = "windows")]
use std::io::prelude::*;
#[cfg(target_os = "windows")]
use std::io::Seek;
#[cfg(target_os = "windows")]
use std::iter::Iterator;
#[cfg(target_os = "windows")]
use walkdir::DirEntry;
#[cfg(target_os = "windows")]
use zip::write::FileOptions;

/// Creates a `.tar.gz` file from the given directory (placing the new file
/// within the given directory's parent directory), then deletes the original
/// directory and returns the path to the new file.
pub fn tar_and_gzip_dir<P: AsRef<Path>>(src_dir: P) -> crate::Result<PathBuf> {
  let src_dir = src_dir.as_ref();
  let dest_path = src_dir.with_extension("tar.gz");
  let dest_file = common::create_file(&dest_path)?;
  let gzip_encoder = gzip::Encoder::new(dest_file)?;
  let gzip_encoder = create_tar_from_src(src_dir, gzip_encoder)?;
  let mut dest_file = gzip_encoder.finish().into_result()?;
  dest_file.flush()?;
  Ok(dest_path)
}

pub fn tar_and_gzip_to<P: AsRef<Path>>(src_dir: P, dst_file: P) -> crate::Result<PathBuf> {
  let src_dir = src_dir.as_ref();
  let dest_path = dst_file.as_ref().to_path_buf();
  let dest_file = common::create_file(&dest_path)?;
  let gzip_encoder = gzip::Encoder::new(dest_file)?;

  let gzip_encoder = create_tar_from_src(src_dir, gzip_encoder)?;
  let mut dest_file = gzip_encoder.finish().into_result()?;
  dest_file.flush()?;
  Ok(dest_path)
}

/// Writes a tar file to the given writer containing the given directory.
/// /tmp/test /tmp/archive.tar.gz
fn create_tar_from_src<P: AsRef<Path>, W: Write>(src_dir: P, dest_file: W) -> crate::Result<W> {
  let src_dir = src_dir.as_ref();
  let mut tar_builder = tar::Builder::new(dest_file);

  // validate source type
  let file_type = fs::metadata(src_dir).expect("Can't read source directory");
  // if it's a file don't need to walkdir
  if file_type.is_file() {
    let mut src_file = fs::File::open(src_dir)?;
    let file_name = src_dir
      .file_name()
      .expect("Can't extract file name from path");

    tar_builder.append_file(file_name, &mut src_file)?;
  } else {
    for entry in WalkDir::new(&src_dir) {
      let entry = entry?;
      let src_path = entry.path();
      if src_path == src_dir {
        continue;
      }

      // todo(lemarier): better error catching
      // We add the .parent() because example if we send a path
      // /dev/src-tauri/target/debug/bundle/osx/app.app
      // We need a tar with app.app/<...> (source root folder should be included)
      let dest_path = src_path.strip_prefix(&src_dir.parent().expect(""))?;
      if entry.file_type().is_dir() {
        tar_builder.append_dir(dest_path, src_path)?;
      } else {
        let mut src_file = fs::File::open(src_path)?;
        tar_builder.append_file(dest_path, &mut src_file)?;
      }
    }
  }
  let dest_file = tar_builder.into_inner()?;
  Ok(dest_file)
}

#[cfg(target_os = "windows")]
pub fn zip_dir(src_dir: &PathBuf, dst_file: &PathBuf) -> crate::Result<PathBuf> {
  let parent_dir = dst_file.parent().expect("No data in parent");
  fs::create_dir_all(parent_dir)?;
  let file = common::create_file(dst_file)?;

  let walkdir = WalkDir::new(src_dir);
  let it = walkdir.into_iter();

  zip_it(&mut it.filter_map(|e| e.ok()), src_dir, file)?;

  Ok(dst_file.to_owned())
}

#[cfg(target_os = "windows")]
fn zip_it<T>(
  it: &mut dyn Iterator<Item = DirEntry>,
  prefix: &PathBuf,
  writer: T,
) -> zip::result::ZipResult<()>
where
  T: Write + Seek,
{
  let mut zip = zip::ZipWriter::new(writer);
  let options = FileOptions::default()
    .compression_method(zip::CompressionMethod::Deflated)
    .unix_permissions(0o755);

  let mut buffer = Vec::new();
  for entry in it {
    let path = entry.path();
    let name = path.strip_prefix(prefix).unwrap();

    // Write file or directory explicitly
    // Some unzip tools unzip files with directory paths correctly, some do not!
    if path.is_file() {
      println!("adding file {:?} as {:?} ...", path, name);
      zip.start_file_from_path(name, options)?;
      let mut f = File::open(path)?;

      f.read_to_end(&mut buffer)?;
      zip.write_all(&*buffer)?;
      buffer.clear();
    } else if name.as_os_str().len() != 0 {
      // Only if not root! Avoids path spec / warning
      // and mapname conversion failed error on unzip
      println!("adding dir {:?} as {:?} ...", path, name);
      zip.add_directory_from_path(name, options)?;
    }
  }
  zip.finish()?;
  Result::Ok(())
}
