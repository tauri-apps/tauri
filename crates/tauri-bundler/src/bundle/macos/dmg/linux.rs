// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! DMG creation for cross-compiled macOS bundles.
//!
//! The macOS implementation drives `hdiutil` and Finder through AppleScript.
//! Neither exists on Linux, so this path writes a real HFS+ image with the
//! `fstool` crate, patches the Finder metadata that Finder only reads from
//! HFS+, and wraps the image in UDIF with the `udif` crate.
//!
//! An ISO 9660 image with Apple's extensions is not enough: macOS's cd9660
//! driver does not expose the Finder flags, so the volume icon and the window
//! layout stored in `.DS_Store` are both ignored.

use super::super::{app, icon::create_icns_file};
use super::{Bundled, ds_store, hfs};
use crate::{
  PackageType, Settings,
  bundle::{Bundle, settings::Arch},
  error::{Context, ErrorExt},
  utils::fs_utils,
};

use std::{fs, path::Path};

pub fn bundle_project(settings: &Settings, bundles: &[Bundle]) -> crate::Result<Bundled> {
  let app_bundle_paths = if !bundles
    .iter()
    .any(|bundle| bundle.package_type == PackageType::MacOsBundle)
  {
    app::bundle_project(settings)?
  } else {
    Vec::new()
  };

  let output_path = settings.project_out_directory().join("bundle/dmg");
  let package_base_name = format!(
    "{}_{}_{}",
    settings.product_name(),
    settings.version_string(),
    match settings.binary_arch() {
      Arch::X86_64 => "x64",
      Arch::AArch64 => "aarch64",
      Arch::Universal => "universal",
      target => {
        return Err(crate::Error::ArchError(format!(
          "Unsupported architecture: {target:?}"
        )));
      }
    }
  );
  let dmg_name = format!("{package_base_name}.dmg");
  let dmg_path = output_path.join(&dmg_name);
  let bundle_file_name = format!("{}.app", settings.product_name());
  let app_path = settings
    .project_out_directory()
    .join("bundle/macos")
    .join(&bundle_file_name);

  if !app_path.is_dir() {
    return Err(crate::Error::GenericError(format!(
      "failed to find app bundle at {}",
      app_path.display()
    )));
  }

  if output_path.exists() {
    fs::remove_dir_all(&output_path).fs_context("failed to remove old dmg", &output_path)?;
  }
  fs::create_dir_all(&output_path).fs_context("failed to create output directory", &output_path)?;

  if settings.dmg().background.is_some() {
    log::warn!("DMG background images are not supported on Linux; using the default background");
  }
  if settings.license_file().is_some() {
    log::warn!("DMG license/EULA is not supported on Linux");
  }

  log::info!(action = "Bundling"; "{} ({})", dmg_name, dmg_path.display());

  let staging = tempfile::tempdir_in(&output_path)
    .fs_context("failed to create dmg staging directory", &output_path)?;
  let staging_path = staging.path();

  fs_utils::copy_dir(&app_path, &staging_path.join(&bundle_file_name))
    .context("failed to copy app bundle into dmg staging directory")?;

  // The /Applications link is what makes the mounted image drag-installable.
  std::os::unix::fs::symlink("/Applications", staging_path.join("Applications")).fs_context(
    "failed to create Applications link",
    staging_path.join("Applications"),
  )?;

  let volume_icon =
    create_icns_file(&output_path, settings).context("failed to create the dmg volume icon")?;
  if let Some(volume_icon) = &volume_icon {
    // `bundle_dmg.sh` also runs `SetFile -a C` on the volume root so Finder
    // picks this up. That flag lives in the root folder's catalog record,
    // which no Linux tool sets, so it is patched in after the copy.
    fs_utils::copy_file(volume_icon, &staging_path.join(".VolumeIcon.icns"))
      .context("failed to copy the dmg volume icon")?;
  }

  // Without this file Finder shows a plain window with the icons in whatever
  // position it likes; it is what carries the window size, icon size, and the
  // app/Applications coordinates.
  let dmg_settings = settings.dmg();
  ds_store::write(
    staging_path,
    &ds_store::Layout {
      app_name: &bundle_file_name,
      window_size: (
        dmg_settings.window_size.width,
        dmg_settings.window_size.height,
      ),
      window_position: dmg_settings.window_position.clone(),
      app_position: dmg_settings.app_position.clone(),
      application_folder_position: dmg_settings.application_folder_position.clone(),
    },
  )?;

  let image_path = output_path.join("dmg-staging.hfs");
  create_volume(settings, staging_path, &image_path)?;
  // The Finder flags that drive the volume icon live in the HFS+ catalog, so
  // they are patched after the files are in place.
  if let Err(error) = hfs::finalize(&image_path, volume_icon.is_some()) {
    log::warn!("failed to set the dmg volume Finder metadata: {error}");
  }

  log::info!(action = "Bundling"; "{} (UDIF)", dmg_path.display());
  udif::create_from_file(&dmg_path, &image_path, "Apple_HFS").map_err(|error| {
    crate::Error::GenericError(format!("failed to write {}: {error}", dmg_path.display()))
  })?;

  fs::remove_file(&image_path).fs_context("failed to remove dmg staging image", &image_path)?;

  if !dmg_path.is_file() {
    return Err(crate::Error::GenericError(format!(
      "failed to create {}",
      dmg_path.display()
    )));
  }

  Ok(Bundled {
    dmg: vec![dmg_path],
    app: app_bundle_paths,
  })
}

/// Create an HFS+ volume and copy the staged files into it.
fn create_volume(settings: &Settings, staging: &Path, image_path: &Path) -> crate::Result<()> {
  // HFS+ volumes cannot be resized after the fact, so leave room for the
  // catalog B-tree and the allocation bitmap on top of the payload.
  let payload = total_size(staging)?;
  let image_len = (payload * 12 / 10 + 16 * 1024 * 1024).max(32 * 1024 * 1024);
  log::info!(action = "Bundling"; "{} (HFS+)", image_path.display());

  let mut device = fstool::block::FileBackend::create(image_path, image_len)
    .hfs_context("failed to create the dmg volume image", image_path)?;
  let options = fstool::fs::hfs_plus::FormatOpts {
    volume_name: volume_name(settings.product_name()),
    catalog_nodes: catalog_nodes(staging)?,
    ..Default::default()
  };
  let mut volume = fstool::fs::hfs_plus::HfsPlus::format(&mut device, &options)
    .hfs_context("failed to format the dmg volume", image_path)?;

  copy_tree(&mut volume, &mut device, staging, image_path)?;
  volume
    .flush(&mut device)
    .hfs_context("failed to write the dmg volume", image_path)?;

  Ok(())
}

/// Copy the staging tree into the volume.
fn copy_tree(
  volume: &mut fstool::fs::hfs_plus::HfsPlus,
  device: &mut fstool::block::FileBackend,
  staging: &Path,
  image_path: &Path,
) -> crate::Result<()> {
  // `WalkDir` yields parents before children, so directories can be created
  // as they are visited.
  for entry in walkdir::WalkDir::new(staging)
    .min_depth(1)
    .sort_by_file_name()
  {
    let entry = entry?;
    let path = entry.path();
    let relative = path.strip_prefix(staging)?;
    let destination = format!("/{}", relative.to_string_lossy().replace('\\', "/"));
    let metadata = entry.metadata()?;
    let mode = unix_mode(&metadata);
    let mtime = mtime(&metadata);

    if entry.file_type().is_symlink() {
      let target = fs::read_link(path)?;
      volume
        .create_symlink(
          device,
          &destination,
          &target.to_string_lossy(),
          mode,
          0,
          0,
          mtime,
        )
        .hfs_context("failed to create a symlink in the dmg volume", image_path)?;
    } else if entry.file_type().is_dir() {
      volume
        .create_dir(device, &destination, mode, 0, 0, mtime)
        .hfs_context("failed to create a directory in the dmg volume", image_path)?;
    } else if entry.file_type().is_file() {
      let mut source = fs::File::open(path)?;
      volume
        .create_file(
          device,
          &destination,
          &mut source,
          metadata.len(),
          mode,
          0,
          0,
          mtime,
        )
        .hfs_context("failed to copy a file into the dmg volume", image_path)?;
    }
  }

  Ok(())
}

fn unix_mode(metadata: &fs::Metadata) -> u16 {
  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;
    (metadata.permissions().mode() & 0o7777) as u16
  }
  #[cfg(not(unix))]
  {
    0o755
  }
}

fn mtime(metadata: &fs::Metadata) -> u32 {
  metadata
    .modified()
    .ok()
    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
    .map(|duration| duration.as_secs().min(u32::MAX as u64) as u32)
    .unwrap_or(0)
}

/// The HFS+ catalog B-tree is fixed at format time, so it has to be sized for
/// every directory and file in the bundle (plus their thread records).
fn catalog_nodes(staging: &Path) -> crate::Result<u32> {
  let mut entries = 0u64;
  for entry in walkdir::WalkDir::new(staging).min_depth(1) {
    entry?;
    entries += 1;
  }
  // Two records per entry (record + thread) and roughly 8 records per 8 KiB
  // node; the writer's default of 32 nodes covers ~130 entries.
  let needed = entries.saturating_mul(2).div_ceil(6) + 8;
  Ok(u32::try_from(needed).unwrap_or(u32::MAX).max(32))
}

fn total_size(directory: &Path) -> crate::Result<u64> {
  let mut total = 0;
  for entry in walkdir::WalkDir::new(directory) {
    let entry = entry?;
    if entry.file_type().is_file() {
      total += entry.metadata()?.len();
    }
  }
  Ok(total)
}

fn volume_name(product_name: &str) -> String {
  // HFS+ stores the name as UTF-16 and caps it at 255 characters.
  product_name.chars().take(255).collect()
}

/// Attach the image path to an `fstool` error.
trait HfsContext<T> {
  fn hfs_context(self, message: &str, image: &Path) -> crate::Result<T>;
}

impl<T> HfsContext<T> for Result<T, fstool::Error> {
  fn hfs_context(self, message: &str, image: &Path) -> crate::Result<T> {
    self.map_err(|error| {
      crate::Error::GenericError(format!("{message} ({}): {error}", image.display()))
    })
  }
}

#[cfg(test)]
mod tests {
  use super::{bundle_project, volume_name};
  use crate::{
    PackageType, Settings,
    bundle::{
      Bundle, BundleBinary, BundleSettings, MacOsSettings, PackageSettings, SettingsBuilder,
    },
  };
  use std::{fs, path::Path};

  fn settings(project_dir: &Path, no_sign: bool) -> Settings {
    SettingsBuilder::new()
      .project_out_directory(project_dir)
      .package_settings(PackageSettings {
        product_name: "Test".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: MacOsSettings::default(),
        ..Default::default()
      })
      .target("aarch64-apple-darwin".into())
      .no_sign(no_sign)
      .build()
      .expect("failed to build settings")
  }

  fn write_binary(project_dir: &Path) {
    let executable = project_dir.join("Test");
    fs::create_dir_all(executable.parent().unwrap()).unwrap();
    fs::write(&executable, b"#!/bin/sh\n").unwrap();
  }

  fn settings_with_icon(project_dir: &Path) -> Settings {
    let icon = project_dir.join("icon.icns");
    fs::write(&icon, b"icns-not-really").unwrap();

    SettingsBuilder::new()
      .project_out_directory(project_dir)
      .package_settings(PackageSettings {
        product_name: "Test".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: MacOsSettings::default(),
        icon: Some(vec![icon.to_string_lossy().into_owned()]),
        ..Default::default()
      })
      .target("aarch64-apple-darwin".into())
      .no_sign(true)
      .build()
      .expect("failed to build settings")
  }

  #[test]
  fn volume_name_is_passed_through() {
    assert_eq!(volume_name("TestApp"), "TestApp");
    assert_eq!(volume_name("网络工具"), "网络工具");
    assert_eq!(volume_name(&"a".repeat(300)).chars().count(), 255);
  }

  #[test]
  fn builds_dmg_from_an_existing_app_bundle() {
    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());
    let app_path = project_dir.path().join("bundle/macos/Test.app");
    fs::create_dir_all(app_path.join("Contents/MacOS")).unwrap();
    fs::write(app_path.join("Contents/MacOS/Test"), b"#!/bin/sh\n").unwrap();
    let bundles = vec![Bundle {
      package_type: PackageType::MacOsBundle,
      bundle_paths: vec![app_path],
    }];

    let bundled = bundle_project(&settings(project_dir.path(), true), &bundles).unwrap();

    assert_eq!(bundled.dmg.len(), 1);
    let dmg_path = &bundled.dmg[0];
    assert_eq!(
      dmg_path.file_name().unwrap().to_string_lossy(),
      "Test_0.1.0_aarch64.dmg"
    );
    let bytes = fs::read(dmg_path).unwrap();
    assert!(!bytes.is_empty());
    // UDIF images end with the `koly` trailer.
    assert_eq!(&bytes[bytes.len() - 512..bytes.len() - 508], b"koly");

    // The intermediate HFS+ image must not leak into the output directory.
    assert!(!dmg_path.with_file_name("dmg-staging.hfs").exists());
  }

  #[test]
  fn builds_app_bundle_and_dmg_from_scratch() {
    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());

    let settings = SettingsBuilder::new()
      .project_out_directory(project_dir.path())
      .package_settings(PackageSettings {
        product_name: "Test".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        macos: MacOsSettings::default(),
        ..Default::default()
      })
      .binaries(vec![BundleBinary::with_path(
        "Test".into(),
        true,
        Some("src/main.rs".into()),
      )])
      .target("aarch64-apple-darwin".into())
      .no_sign(true)
      .build()
      .expect("failed to build settings");

    // No prebuilt bundles: the dmg path has to build the app itself.
    let bundled = bundle_project(&settings, &[]).unwrap();

    assert_eq!(bundled.app.len(), 1);
    let app_path = &bundled.app[0];
    assert!(app_path.join("Contents/Info.plist").is_file());
    assert!(app_path.join("Contents/MacOS/Test").is_file());
    assert_eq!(bundled.dmg.len(), 1);
  }

  #[test]
  fn marks_the_volume_root_when_an_icon_is_bundled() {
    let project_dir = tempfile::tempdir().unwrap();
    write_binary(project_dir.path());
    let app_path = project_dir.path().join("bundle/macos/Test.app");
    fs::create_dir_all(app_path.join("Contents/MacOS")).unwrap();
    let executable = app_path.join("Contents/MacOS/Test");
    fs::write(&executable, b"#!/bin/sh\n").unwrap();
    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
      // Frameworks link to their siblings; the link must stay a link.
      std::os::unix::fs::symlink("Test", app_path.join("Contents/MacOS/TestLink")).unwrap();
    }
    let bundles = vec![Bundle {
      package_type: PackageType::MacOsBundle,
      bundle_paths: vec![app_path],
    }];

    let settings = settings_with_icon(project_dir.path());
    let bundled = bundle_project(&settings, &bundles).unwrap();

    // Read the image back out of the UDIF container without any external tool.
    let mut archive = udif::DmgArchive::open(&bundled.dmg[0]).unwrap();
    let bytes = archive.extract_main_partition().unwrap();
    assert_eq!(&bytes[1024..1026], b"H+");
    let flags = super::hfs::root_folder_flags(&bytes).unwrap();
    assert_eq!(flags & 0x0400, 0x0400);

    // The executable bit and the symlink have to survive the copy.
    let image = project_dir.path().join("verify.img");
    fs::write(&image, &bytes).unwrap();
    let mut device = fstool::block::FileBackend::open(&image).unwrap();
    let volume = fstool::fs::hfs_plus::HfsPlus::open(&mut device).unwrap();

    let entries = volume
      .list_path(&mut device, "/Test.app/Contents/MacOS")
      .unwrap();
    let names: Vec<_> = entries.iter().map(|entry| entry.name.as_str()).collect();
    assert!(names.contains(&"Test"), "{names:?}");
    assert!(names.contains(&"TestLink"), "{names:?}");

    let link = volume
      .read_symlink_target_path(&mut device, "/Test.app/Contents/MacOS/TestLink")
      .unwrap();
    assert_eq!(link, "Test");

    let applications = volume
      .read_symlink_target_path(&mut device, "/Applications")
      .unwrap();
    assert_eq!(applications, "/Applications");

    assert!(
      volume
        .list_path(&mut device, "/")
        .unwrap()
        .iter()
        .any(|entry| entry.name == ".DS_Store")
    );
  }
}
