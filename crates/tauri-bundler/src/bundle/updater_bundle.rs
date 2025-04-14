// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::{
    windows::{
      NSIS_OUTPUT_FOLDER_NAME, NSIS_UPDATER_OUTPUT_FOLDER_NAME, WIX_OUTPUT_FOLDER_NAME,
      WIX_UPDATER_OUTPUT_FOLDER_NAME,
    },
    Bundle,
  },
  utils::fs_utils,
  Settings,
};
use tauri_utils::display_path;

use std::{
  fs::{self, File},
  io::prelude::*,
  path::{Path, PathBuf},
};

use anyhow::Context;
use zip::write::SimpleFileOptions;

// Build update
pub fn bundle_project(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  let target_os = settings
    .target()
    .split('-')
    .nth(2)
    .unwrap_or(std::env::consts::OS)
    .replace("darwin", "macos");

  if target_os == "windows" {
    return bundle_update_windows(settings, bundles);
  }

  #[cfg(target_os = "macos")]
  return bundle_update_macos(bundles);
  #[cfg(target_os = "linux")]
  return bundle_update_linux(bundles);

  #[cfg(not(any(target_os = "macos", target_os = "linux")))]
  {
    log::error!("Current platform does not support updates");
    Ok(vec![])
  }
}

// Create simple update-macos.tar.gz
// This is the Mac OS App packaged
#[cfg(target_os = "macos")]
fn bundle_update_macos(bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  use std::ffi::OsStr;

  // find our .app or rebuild our bundle
  if let Some(source_path) = bundles
    .iter()
    .filter(|bundle| bundle.package_type == crate::PackageType::MacOsBundle)
    .find_map(|bundle| {
      bundle
        .bundle_paths
        .iter()
        .find(|path| path.extension() == Some(OsStr::new("app")))
    })
  {
    // add .tar.gz to our path
    let osx_archived = format!("{}.tar.gz", source_path.display());
    let osx_archived_path = PathBuf::from(&osx_archived);

    // Create our gzip file (need to send parent)
    // as we walk the source directory (source isnt added)
    create_tar(source_path, &osx_archived_path)
      .with_context(|| "Failed to tar.gz update directory")?;

    log::info!(action = "Bundling"; "{} ({})", osx_archived, display_path(&osx_archived_path));

    Ok(vec![osx_archived_path])
  } else {
    Err(crate::Error::UnableToFindProject)
  }
}

// Create simple update-linux_<arch>.tar.gz
// Including the AppImage
// Right now in linux we hot replace the bin and request a restart
// No assets are replaced
#[cfg(target_os = "linux")]
fn bundle_update_linux(bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  use std::ffi::OsStr;

  // build our app actually we support only appimage on linux
  if let Some(source_path) = bundles
    .iter()
    .filter(|bundle| bundle.package_type == crate::PackageType::AppImage)
    .find_map(|bundle| {
      bundle
        .bundle_paths
        .iter()
        .find(|path| path.extension() == Some(OsStr::new("AppImage")))
    })
  {
    // add .tar.gz to our path
    let appimage_archived = format!("{}.tar.gz", source_path.display());
    let appimage_archived_path = PathBuf::from(&appimage_archived);

    // Create our gzip file
    create_tar(source_path, &appimage_archived_path)
      .with_context(|| "Failed to tar.gz update directory")?;

    log::info!(action = "Bundling"; "{} ({})", appimage_archived, display_path(&appimage_archived_path));

    Ok(vec![appimage_archived_path])
  } else {
    Err(crate::Error::UnableToFindProject)
  }
}

// Create simple update-win_<arch>.zip
// Including the binary as root
// Right now in windows we hot replace the bin and request a restart
// No assets are replaced
fn bundle_update_windows(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Vec<PathBuf>> {
  use crate::bundle::settings::WebviewInstallMode;
  #[cfg(target_os = "windows")]
  use crate::bundle::windows::msi;
  use crate::bundle::windows::nsis;
  use crate::PackageType;

  // find our installers or rebuild
  let mut bundle_paths = Vec::new();
  let mut rebuild_installers = || -> crate::Result<()> {
    for bundle in bundles {
      match bundle.package_type {
        #[cfg(target_os = "windows")]
        PackageType::WindowsMsi => bundle_paths.extend(msi::bundle_project(settings, true)?),
        PackageType::Nsis => bundle_paths.extend(nsis::bundle_project(settings, true)?),
        _ => {}
      };
    }
    Ok(())
  };

  if matches!(
    settings.windows().webview_install_mode,
    WebviewInstallMode::OfflineInstaller { .. } | WebviewInstallMode::EmbedBootstrapper { .. }
  ) {
    rebuild_installers()?;
  } else {
    let paths = bundles
      .iter()
      .filter(|bundle| {
        matches!(
          bundle.package_type,
          PackageType::WindowsMsi | PackageType::Nsis
        )
      })
      .flat_map(|bundle| bundle.bundle_paths.clone())
      .collect::<Vec<_>>();

    // we expect our installer files to be on `bundle_paths`
    if paths.is_empty() {
      rebuild_installers()?;
    } else {
      bundle_paths.extend(paths);
    }
  };

  let mut installers_archived_paths = Vec::new();
  for source_path in bundle_paths {
    // add .zip to our path
    let (archived_path, bundle_name) =
      source_path
        .components()
        .fold((PathBuf::new(), String::new()), |(mut p, mut b), c| {
          if let std::path::Component::Normal(name) = c {
            if let Some(name) = name.to_str() {
              // installers bundled for updater should be put in a directory named `${bundle_name}-updater`
              if name == WIX_UPDATER_OUTPUT_FOLDER_NAME || name == NSIS_UPDATER_OUTPUT_FOLDER_NAME {
                b = name.strip_suffix("-updater").unwrap().to_string();
                p.push(&b);
                return (p, b);
              }

              if name == WIX_OUTPUT_FOLDER_NAME || name == NSIS_OUTPUT_FOLDER_NAME {
                b = name.to_string();
              }
            }
          }
          p.push(c);
          (p, b)
        });
    let archived_path = archived_path.with_extension(format!("{}.zip", bundle_name));

    log::info!(action = "Bundling"; "{}", display_path(&archived_path));

    // Create our gzip file
    create_zip(&source_path, &archived_path).with_context(|| "Failed to zip update bundle")?;

    installers_archived_paths.push(archived_path);
  }

  Ok(installers_archived_paths)
}

pub fn create_zip(src_file: &Path, dst_file: &Path) -> crate::Result<PathBuf> {
  let parent_dir = dst_file.parent().expect("No data in parent");
  fs::create_dir_all(parent_dir)?;
  let writer = fs_utils::create_file(dst_file)?;

  let file_name = src_file
    .file_name()
    .expect("Can't extract file name from path");

  let mut zip = zip::ZipWriter::new(writer);
  let options = SimpleFileOptions::default()
    .compression_method(zip::CompressionMethod::Stored)
    .unix_permissions(0o755);

  zip.start_file(file_name.to_string_lossy(), options)?;
  let mut f = File::open(src_file)?;
  let mut buffer = Vec::new();
  f.read_to_end(&mut buffer)?;
  zip.write_all(&buffer)?;
  buffer.clear();

  Ok(dst_file.to_owned())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn create_tar(src_dir: &Path, dest_path: &Path) -> crate::Result<PathBuf> {
  use flate2::{write::GzEncoder, Compression};

  let dest_file = fs_utils::create_file(dest_path)?;
  let gzip_encoder = GzEncoder::new(dest_file, Compression::default());

  let gzip_encoder = create_tar_from_src(src_dir, gzip_encoder)?;

  let mut dest_file = gzip_encoder.finish()?;
  dest_file.flush()?;
  Ok(dest_path.to_owned())
}

#[cfg(target_os = "macos")]
fn create_tar_from_src<P: AsRef<Path>, W: Write>(src_dir: P, dest_file: W) -> crate::Result<W> {
  let src_dir = src_dir.as_ref();
  let mut builder = tar::Builder::new(dest_file);
  builder.follow_symlinks(false);
  builder.append_dir_all(src_dir.file_name().expect("Path has no file_name"), src_dir)?;
  builder.into_inner().map_err(Into::into)
}

#[cfg(target_os = "linux")]
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
    for entry in walkdir::WalkDir::new(src_dir) {
      let entry = entry?;
      let src_path = entry.path();
      if src_path == src_dir {
        continue;
      }

      // We add the .parent() because example if we send a path
      // /dev/src-tauri/target/debug/bundle/osx/app.app
      // We need a tar with app.app/<...> (source root folder should be included)
      // safe to unwrap: the path has a parent
      let dest_path = src_path.strip_prefix(src_dir.parent().unwrap())?;
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
