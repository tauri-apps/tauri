// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::bundle::Settings;
use crate::utils::{self, CommandExt, fs_utils};
use std::{
  cmp::min,
  ffi::OsStr,
  fs::{self, File},
  io::{self, BufWriter},
  path::{Path, PathBuf},
  process::Command,
};

use image::GenericImageView;

// Given a list of icon files, try to produce an ICNS file in the out_dir
// and return the path to it.  Returns `Ok(None)` if no usable icons
// were provided.
pub fn create_icns_file(out_dir: &Path, settings: &Settings) -> crate::Result<Option<PathBuf>> {
  if settings.icon_files().count() == 0 {
    return Ok(None);
  }

  // If one of the icon files is already an ICNS file, just use that.
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;
    if icon_path.extension() == Some(OsStr::new("icns")) {
      let mut dest_path = out_dir.to_path_buf();
      dest_path.push(icon_path.file_name().expect("Could not get icon filename"));
      fs_utils::copy_file(&icon_path, &dest_path)?;
      return Ok(Some(dest_path));
    }
  }

  // Otherwise, read available images and pack them into a new ICNS file.
  let mut family = icns::IconFamily::new();

  fn add_icon_to_family(
    icon: image::DynamicImage,
    density: u32,
    family: &mut icns::IconFamily,
  ) -> io::Result<()> {
    // Try to add this image to the icon family.  Ignore images whose sizes
    // don't map to any ICNS icon type; print warnings and skip images that
    // fail to encode.
    match icns::IconType::from_pixel_size_and_density(icon.width(), icon.height(), density) {
      Some(icon_type) => {
        if !family.has_icon_with_type(icon_type) {
          let icon = make_icns_image(icon)?;
          family.add_icon_with_type(&icon, icon_type)?;
        }
        Ok(())
      }
      None => Err(io::Error::new(
        io::ErrorKind::InvalidData,
        "No matching IconType",
      )),
    }
  }

  let mut images_to_resize: Vec<(image::DynamicImage, u32, u32)> = vec![];
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;

    if icon_path.extension().is_some_and(|ext| ext == "car") {
      continue;
    }

    let icon = image::open(&icon_path)?;
    let density = if utils::is_retina(&icon_path) { 2 } else { 1 };
    let (w, h) = icon.dimensions();
    let orig_size = min(w, h);
    let next_size_down = 2f32.powf((orig_size as f32).log2().floor()) as u32;
    if orig_size > next_size_down {
      images_to_resize.push((icon, next_size_down, density));
    } else {
      add_icon_to_family(icon, density, &mut family)?;
    }
  }

  for (icon, next_size_down, density) in images_to_resize {
    let icon = icon.resize_exact(
      next_size_down,
      next_size_down,
      image::imageops::FilterType::Lanczos3,
    );
    add_icon_to_family(icon, density, &mut family)?;
  }

  if !family.is_empty() {
    fs::create_dir_all(out_dir)?;
    let mut dest_path = out_dir.to_path_buf();
    dest_path.push(settings.product_name());
    dest_path.set_extension("icns");
    let icns_file = BufWriter::new(File::create(&dest_path)?);
    family.write(icns_file)?;
    Ok(Some(dest_path))
  } else {
    Err(crate::Error::GenericError(
      "No usable Icon files found".to_owned(),
    ))
  }
}

// Converts an image::DynamicImage into an icns::Image.
fn make_icns_image(img: image::DynamicImage) -> io::Result<icns::Image> {
  let pixel_format = match img.color() {
    image::ColorType::Rgba8 => icns::PixelFormat::RGBA,
    image::ColorType::Rgb8 => icns::PixelFormat::RGB,
    image::ColorType::La8 => icns::PixelFormat::GrayAlpha,
    image::ColorType::L8 => icns::PixelFormat::Gray,
    _ => {
      let msg = format!("unsupported ColorType: {:?}", img.color());
      return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
    }
  };
  icns::Image::from_data(pixel_format, img.width(), img.height(), img.into_bytes())
}

/// Creates an Assets.car file from a .icon file if there are any in the settings.
/// Uses an existing Assets.car file if it exists in the settings.
/// Returns the path to the Assets.car file.
pub fn create_assets_car_file(
  out_dir: &Path,
  settings: &Settings,
) -> crate::Result<Option<PathBuf>> {
  let Some(icons) = settings.icons() else {
    return Ok(None);
  };
  // If one of the icon files is already a CAR file, just use that.
  let mut icon_composer_icon_path = None;
  for icon in icons {
    let icon_path = Path::new(&icon).to_path_buf();
    if icon_path.extension() == Some(OsStr::new("car")) {
      let dest_path = out_dir.join("Assets.car");
      fs_utils::copy_file(&icon_path, &dest_path)?;
      return Ok(Some(dest_path));
    }

    if icon_path.extension() == Some(OsStr::new("icon")) {
      icon_composer_icon_path.replace(icon_path);
    }
  }

  let Some(icon_composer_icon_path) = icon_composer_icon_path else {
    return Ok(None);
  };

  // Check actool version - must be >= 26
  if let Some(version) = get_actool_version() {
    // Parse the major version number (before the dot)
    let major_version: Option<u32> = version.split('.').next().and_then(|s| s.parse().ok());

    if let Some(major) = major_version {
      if major < 26 {
        log::error!(
          "actool version is less than 26, skipping Assets.car file creation. Please update Xcode to 26 or above and try again."
        );
        return Ok(None);
      }
    } else {
      // If we can't parse the version, return None to be safe
      log::error!("failed to parse actool version, skipping Assets.car file creation");
      return Ok(None);
    }
  } else {
    log::error!("failed to get actool version, skipping Assets.car file creation");
    // If we can't get the version, return None to be safe
    return Ok(None);
  }

  // Create a temporary directory for actool work
  let temp_dir = tempfile::tempdir()
    .map_err(|e| crate::Error::GenericError(format!("failed to create temp dir: {e}")))?;

  let icon_dest_path = temp_dir.path().join("Icon.icon");
  let output_path = temp_dir.path().join("out");

  // Copy the input .icon directory to the temp directory
  if icon_composer_icon_path.is_dir() {
    fs_utils::copy_dir(&icon_composer_icon_path, &icon_dest_path)?;
  } else {
    return Err(crate::Error::GenericError(format!(
      "{} must be a directory",
      icon_composer_icon_path.display()
    )));
  }

  // Create the output directory
  fs::create_dir_all(&output_path)?;

  // Run actool command
  let mut cmd = Command::new("actool");
  cmd.arg(&icon_dest_path);
  cmd.arg("--compile");
  cmd.arg(&output_path);
  cmd.arg("--output-format");
  cmd.arg("human-readable-text");
  cmd.arg("--notices");
  cmd.arg("--warnings");
  cmd.arg("--output-partial-info-plist");
  cmd.arg(output_path.join("assetcatalog_generated_info.plist"));
  cmd.arg("--app-icon");
  cmd.arg("Icon");
  cmd.arg("--include-all-app-icons");
  cmd.arg("--accent-color");
  cmd.arg("AccentColor");
  cmd.arg("--enable-on-demand-resources");
  cmd.arg("NO");
  cmd.arg("--development-region");
  cmd.arg("en");
  cmd.arg("--target-device");
  cmd.arg("mac");
  cmd.arg("--minimum-deployment-target");
  cmd.arg("26.0");
  cmd.arg("--platform");
  cmd.arg("macosx");

  cmd.output_ok()?;

  let assets_car_path = output_path.join("Assets.car");
  if !assets_car_path.exists() {
    return Err(crate::Error::GenericError(
      "actool did not generate Assets.car file".to_owned(),
    ));
  }

  // copy to out_dir
  fs_utils::copy_file(&assets_car_path, &out_dir.join("Assets.car"))?;

  Ok(Some(out_dir.join("Assets.car")))
}

#[derive(serde::Deserialize)]
struct AssetsCarInfo {
  #[serde(rename = "AssetType", default)]
  asset_type: String,
  #[serde(rename = "Name", default)]
  name: String,
}

pub fn app_icon_name_from_assets_car(assets_car_path: &Path) -> Option<String> {
  let Ok(output) = Command::new("assetutil")
    .arg("--info")
    .arg(assets_car_path)
    .output_ok()
    .inspect_err(|e| log::error!("Failed to get app icon name from Assets.car file: {e}"))
  else {
    return None;
  };

  let output = String::from_utf8(output.stdout).ok()?;
  let assets_car_info: Vec<AssetsCarInfo> = serde_json::from_str(&output)
    .inspect_err(|e| log::error!("Failed to parse Assets.car file info: {e}"))
    .ok()?;
  assets_car_info
    .iter()
    .find(|info| info.asset_type == "Icon Image")
    .map(|info| info.name.clone())
}

/// Returns the actool short bundle version by running `actool --version --output-format=human-readable-text`.
/// Returns `None` if the command fails or the output cannot be parsed.
pub fn get_actool_version() -> Option<String> {
  let Ok(output) = Command::new("actool")
    .arg("--version")
    .arg("--output-format=human-readable-text")
    .output_ok()
    .inspect_err(|e| log::error!("Failed to get actool version: {e}"))
  else {
    return None;
  };

  let output = String::from_utf8(output.stdout).ok()?;
  parse_actool_version(&output)
}

fn parse_actool_version(output: &str) -> Option<String> {
  // The output format is:
  // /* com.apple.actool.version */
  // bundle-version: 24411
  // short-bundle-version: 26.1
  for line in output.lines() {
    let line = line.trim();
    if let Some(version) = line.strip_prefix("short-bundle-version:") {
      return Some(version.trim().to_string());
    }
  }

  None
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_parse_actool_version() {
    let output = r#"/* com.apple.actool.version */
some other line
bundle-version: 24411
short-bundle-version: 26.1
another line
"#;

    let version = parse_actool_version(output).expect("Failed to parse version");
    assert_eq!(version, "26.1");
  }

  #[test]
  fn test_parse_actool_version_missing_fields() {
    let output = r#"/* com.apple.actool.version */
bundle-version: 24411
"#;

    assert!(parse_actool_version(output).is_none());
  }

  #[test]
  fn test_parse_actool_version_empty() {
    assert!(parse_actool_version("").is_none());
  }
}
