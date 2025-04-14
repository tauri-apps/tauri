// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::bundle::Settings;
use crate::utils::{self, fs_utils};
use std::{
  cmp::min,
  ffi::OsStr,
  fs::{self, File},
  io::{self, BufWriter},
  path::{Path, PathBuf},
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
