// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// An iOS package is laid out like:
//
// Foobar.app         # Actually a directory
//     Foobar             # The main binary executable of the app
//     Info.plist         # An XML file containing the app's metadata
//     ...                # Icons and other resource files
//
// See https://developer.apple.com/go/?id=bundle-structure for a full
// explanation.

use crate::{
  utils::{self, fs_utils},
  Settings,
};

use anyhow::Context;
use image::{codecs::png::PngDecoder, GenericImageView, ImageDecoder};

use std::{
  collections::BTreeSet,
  ffi::OsStr,
  fs::{self, File},
  io::{BufReader, Write},
  path::{Path, PathBuf},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the .app was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  log::warn!("iOS bundle support is still experimental.");

  let app_product_name = format!("{}.app", settings.product_name());

  let app_bundle_path = settings
    .project_out_directory()
    .join("bundle/ios")
    .join(&app_product_name);

  log::info!(action = "Bundling"; "{} ({})", app_product_name, app_bundle_path.display());

  if app_bundle_path.exists() {
    fs::remove_dir_all(&app_bundle_path)
      .with_context(|| format!("Failed to remove old {}", app_product_name))?;
  }
  fs::create_dir_all(&app_bundle_path)
    .with_context(|| format!("Failed to create bundle directory at {:?}", app_bundle_path))?;

  for src in settings.resource_files() {
    let src = src?;
    let dest = app_bundle_path.join(tauri_utils::resources::resource_relpath(&src));
    fs_utils::copy_file(&src, &dest)
      .with_context(|| format!("Failed to copy resource file {:?}", src))?;
  }

  let icon_filenames = generate_icon_files(&app_bundle_path, settings)
    .with_context(|| "Failed to create app icons")?;
  generate_info_plist(&app_bundle_path, settings, &icon_filenames)
    .with_context(|| "Failed to create Info.plist")?;

  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    fs_utils::copy_file(&bin_path, &app_bundle_path.join(bin.name()))
      .with_context(|| format!("Failed to copy binary from {:?}", bin_path))?;
  }

  Ok(vec![app_bundle_path])
}

/// Generate the icon files and store them under the `bundle_dir`.
fn generate_icon_files(bundle_dir: &Path, settings: &Settings) -> crate::Result<Vec<String>> {
  let mut filenames = Vec::new();
  {
    let mut get_dest_path = |width: u32, height: u32, is_retina: bool| {
      let filename = format!(
        "icon_{}x{}{}.png",
        width,
        height,
        if is_retina { "@2x" } else { "" }
      );
      let path = bundle_dir.join(&filename);
      filenames.push(filename);
      path
    };
    let mut sizes = BTreeSet::new();
    // Prefer PNG files.
    for icon_path in settings.icon_files() {
      let icon_path = icon_path?;
      if icon_path.extension() != Some(OsStr::new("png")) {
        continue;
      }
      let decoder = PngDecoder::new(BufReader::new(File::open(&icon_path)?))?;
      let width = decoder.dimensions().0;
      let height = decoder.dimensions().1;
      let is_retina = utils::is_retina(&icon_path);
      if !sizes.contains(&(width, height, is_retina)) {
        sizes.insert((width, height, is_retina));
        let dest_path = get_dest_path(width, height, is_retina);
        fs_utils::copy_file(&icon_path, &dest_path)?;
      }
    }
    // Fall back to non-PNG files for any missing sizes.
    for icon_path in settings.icon_files() {
      let icon_path = icon_path?;
      if icon_path.extension() == Some(OsStr::new("png")) {
        continue;
      } else if icon_path.extension() == Some(OsStr::new("icns")) {
        let icon_family = icns::IconFamily::read(File::open(&icon_path)?)?;
        for icon_type in icon_family.available_icons() {
          let width = icon_type.screen_width();
          let height = icon_type.screen_height();
          let is_retina = icon_type.pixel_density() > 1;
          if !sizes.contains(&(width, height, is_retina)) {
            sizes.insert((width, height, is_retina));
            let dest_path = get_dest_path(width, height, is_retina);
            let icon = icon_family.get_icon_with_type(icon_type)?;
            icon.write_png(File::create(dest_path)?)?;
          }
        }
      } else {
        let icon = image::open(&icon_path)?;
        let (width, height) = icon.dimensions();
        let is_retina = utils::is_retina(&icon_path);
        if !sizes.contains(&(width, height, is_retina)) {
          sizes.insert((width, height, is_retina));
          let dest_path = get_dest_path(width, height, is_retina);
          icon.write_to(
            &mut fs_utils::create_file(&dest_path)?,
            image::ImageFormat::Png,
          )?;
        }
      }
    }
  }
  Ok(filenames)
}

/// Generates the Info.plist file
fn generate_info_plist(
  bundle_dir: &Path,
  settings: &Settings,
  icon_filenames: &[String],
) -> crate::Result<()> {
  let file = &mut fs_utils::create_file(&bundle_dir.join("Info.plist"))?;
  writeln!(
    file,
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
          <!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \
          \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
          <plist version=\"1.0\">\n\
          <dict>"
  )?;

  writeln!(
    file,
    "  <key>CFBundleIdentifier</key>\n  <string>{}</string>",
    settings.bundle_identifier()
  )?;
  writeln!(
    file,
    "  <key>CFBundleDisplayName</key>\n  <string>{}</string>",
    settings.product_name()
  )?;
  writeln!(
    file,
    "  <key>CFBundleName</key>\n  <string>{}</string>",
    settings.product_name()
  )?;
  writeln!(
    file,
    "  <key>CFBundleExecutable</key>\n  <string>{}</string>",
    settings.main_binary_name()?
  )?;
  writeln!(
    file,
    "  <key>CFBundleVersion</key>\n  <string>{}</string>",
    settings.version_string()
  )?;
  writeln!(
    file,
    "  <key>CFBundleShortVersionString</key>\n  <string>{}</string>",
    settings.version_string()
  )?;
  writeln!(
    file,
    "  <key>CFBundleDevelopmentRegion</key>\n  <string>en_US</string>"
  )?;

  if !icon_filenames.is_empty() {
    writeln!(file, "  <key>CFBundleIconFiles</key>\n  <array>")?;
    for filename in icon_filenames {
      writeln!(file, "    <string>{}</string>", filename)?;
    }
    writeln!(file, "  </array>")?;
  }
  writeln!(file, "  <key>LSRequiresIPhoneOS</key>\n  <true/>")?;
  writeln!(file, "</dict>\n</plist>")?;
  file.flush()?;
  Ok(())
}
