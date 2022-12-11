// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{helpers::app_paths::tauri_dir, Result};

use std::{
  collections::HashMap,
  fs::{create_dir_all, File},
  io::{BufWriter, Write},
  path::{Path, PathBuf},
};

use anyhow::Context;
use clap::Parser;
use icns::{IconFamily, IconType};
use image::{
  codecs::{
    ico::{IcoEncoder, IcoFrame},
    png::{CompressionType, FilterType as PngFilterType, PngEncoder},
  },
  imageops::FilterType,
  open, ColorType, DynamicImage, ImageEncoder,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IcnsEntry {
  size: u32,
  ostype: String,
}

// #[derive(Debug, Deserialize)]
// struct IconFormatEntry {
//   sizes: Vec<u32>,
//   name: String,
// }

#[derive(Debug, Parser)]
#[clap(about = "Generates various icons for all major platforms")]
pub struct Options {
  // TODO: Confirm 1240px
  /// Path to the source icon (png, 1240x1240px with transparency).
  #[clap(default_value = "./app-icon.png")]
  input: PathBuf,
  /// Output directory.
  /// Default: 'icons' directory next to the tauri.conf.json file.
  #[clap(short, long)]
  output: Option<PathBuf>,

  /// Extra icon sizes.
  #[clap(short, long, use_delimiter = true)]
  extra: Option<Vec<u32>>,
  // /// Generates icons as configured.
  // #[clap(short, long)]
  // config: Option<PathBuf>,
}

pub fn command(options: Options) -> Result<()> {
  let input = options.input;
  let out_dir = options.output.unwrap_or_else(|| tauri_dir().join("icons"));
  let extra_icon_sizes = options
    .extra
    .unwrap_or_else(|| vec![32, 16, 24, 48, 64, 256]);
  create_dir_all(&out_dir).context("Can't create output directory")?;

  // //load config json if possible.
  // let icons_config: Option<HashMap<String, IconFormatEntry>> = match options.config {
  //   Some(config_file) => {
  //     let f = File::open(config_file).context("Cannot read config")?;
  //     serde_json::from_reader(BufReader::new(f)).context("Cannot parse config")?
  //   }
  //   _ => None,
  // };
  // //take png,ico from icons_config
  // let png_config = icons_config.as_ref().and_then(|config| config.get("png"));
  // let ico_config = icons_config.as_ref().and_then(|config| config.get("ico"));

  // Try to read the image as a DynamicImage, convert it to rgba8 and turn it into a DynamicImage again.
  // Both things should be catched by the explicit conversions to rgba8 anyway.
  let source = open(input)
    .context("Can't read and decode source image")?
    .into_rgba8();

  let source = DynamicImage::ImageRgba8(source);

  if source.height() != source.width() {
    panic!("Source image must be square");
  }

  appx(&source, &out_dir).context("Failed to generate appx icons")?;

  icns(&source, &out_dir).context("Failed to generate .icns file")?;

  ico(&source, &out_dir).context("Failed to generate .ico file")?;

  png(&source, &out_dir, extra_icon_sizes).context("Failed to generate png icons")?;

  Ok(())
}

fn appx(source: &DynamicImage, out_dir: &Path) -> Result<()> {
  log::info!(action = "Appx"; "Creating StoreLogo.png");
  resize_and_save_png(source, 50, &out_dir.join("StoreLogo.png"))?;

  for size in [30, 44, 71, 89, 107, 142, 150, 284, 310] {
    let file_name = format!("Square{}x{}Logo.png", size, size);
    log::info!(action = "Appx"; "Creating {}", file_name);

    resize_and_save_png(source, size, &out_dir.join(&file_name))?;
  }

  Ok(())
}

// Main target: macOS
fn icns(source: &DynamicImage, out_dir: &Path) -> Result<()> {
  log::info!(action = "ICNS"; "Creating icon.icns");
  let entries: HashMap<String, IcnsEntry> =
    serde_json::from_slice(include_bytes!("helpers/icns.json")).unwrap();

  let mut family = IconFamily::new();

  for (name, entry) in entries {
    let size = entry.size;
    let mut buf = Vec::new();

    let image = source.resize_exact(size, size, FilterType::Lanczos3);

    write_png(image.as_bytes(), &mut buf, size)?;

    let image = icns::Image::read_png(&buf[..])?;

    family
      .add_icon_with_type(
        &image,
        IconType::from_ostype(entry.ostype.parse().unwrap()).unwrap(),
      )
      .with_context(|| format!("Can't add {} to Icns Family", name))?;
  }

  let mut out_file = BufWriter::new(File::create(out_dir.join("icon.icns"))?);
  family.write(&mut out_file)?;
  out_file.flush()?;

  Ok(())
}

// Generate .ico file with layers for the most common sizes.
// Main target: Windows
fn ico(source: &DynamicImage, out_dir: &Path) -> Result<()> {
  log::info!(action = "ICO"; "Creating icon.ico");
  let mut frames = Vec::new();

  //if no ico config provided, use default
  // let (sizes, icon_name) = match config {
  //   Some(ico_format) => (ico_format.sizes.clone(), ico_format.name.clone()),
  //   None => (vec![32, 16, 24, 48, 64, 256], "icon.ico".to_string()),
  // };
  let sizes = vec![32, 16, 24, 48, 64, 256];
  let icon_name = "icon.ico".to_string();

  for size in sizes {
    let image = source.resize_exact(size, size, FilterType::Lanczos3);

    // Only the 256px layer can be compressed according to the ico specs.
    if size == 256 {
      let mut buf = Vec::new();

      write_png(image.as_bytes(), &mut buf, size)?;

      frames.push(IcoFrame::with_encoded(buf, size, size, ColorType::Rgba8)?)
    } else {
      frames.push(IcoFrame::as_png(
        image.as_bytes(),
        size,
        size,
        ColorType::Rgba8,
      )?);
    }
  }

  let mut out_file = BufWriter::new(File::create(out_dir.join(icon_name))?);
  let encoder = IcoEncoder::new(&mut out_file);
  encoder.encode_images(&frames)?;
  out_file.flush()?;

  Ok(())
}

// Generate .png files in 32x32, 128x128, 256x256, 512x512 (icon.png)
// Main target: Linux
fn png(source: &DynamicImage, out_dir: &Path, mut extra_icon_sizes: Vec<u32>) -> Result<()> {
  //if no config provided, use default
  // let (sizes, _icon_name) = match config {
  //   //TODO: implements icon_name
  //   Some(ico_format) => (ico_format.sizes.clone(), ico_format.name.clone()),
  //   None => (vec![32, 16, 24, 48, 64, 256], "any".to_string()),
  // };
  let mut sizes = vec![32, 16, 24, 48, 64, 256];
  sizes.append(extra_icon_sizes.as_mut());

  for size in sizes {
    let file_name = match size {
      256 => "128x128@2x.png".to_string(),
      512 => "icon.png".to_string(),
      _ => format!("{}x{}.png", size, size),
    };

    log::info!(action = "PNG"; "Creating {}", file_name);
    resize_and_save_png(source, size, &out_dir.join(&file_name))?;
  }

  Ok(())
}

// Resize image and save it to disk.
fn resize_and_save_png(source: &DynamicImage, size: u32, file_path: &Path) -> Result<()> {
  let image = source.resize_exact(size, size, FilterType::Lanczos3);

  let mut out_file = BufWriter::new(File::create(file_path)?);

  write_png(image.as_bytes(), &mut out_file, size)?;

  Ok(out_file.flush()?)
}

// Encode image data as png with compression.
fn write_png<W: Write>(image_data: &[u8], w: W, size: u32) -> Result<()> {
  let encoder = PngEncoder::new_with_quality(w, CompressionType::Best, PngFilterType::Adaptive);
  encoder.write_image(image_data, size, size, ColorType::Rgba8)?;
  Ok(())
}
