// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{helpers::app_paths::tauri_dir, Result};

use std::{
  collections::HashMap,
  fs::{create_dir_all, File},
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
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
  open, DynamicImage, ExtendedColorType, ImageBuffer, ImageEncoder, Rgba,
};
use resvg::{tiny_skia, usvg};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct IcnsEntry {
  size: u32,
  ostype: String,
}

#[derive(Debug)]
struct PngEntry {
  name: String,
  size: u32,
  out_path: PathBuf,
}

struct AndroidEntries {
  foreground: Vec<PngEntry>,
  background: Vec<PngEntry>,
  monochrome: Vec<PngEntry>,
}

#[derive(Deserialize)]
struct Manifest {
  default: String,
  bg_color: Option<String>,
  android_bg: Option<String>,
  android_fg: Option<String>,
  android_monochrome: Option<String>,
}

#[derive(Debug, Parser)]
#[clap(about = "Generate various icons for all major platforms")]
pub struct Options {
  /// Path to the source icon (squared PNG or SVG file with transparency) or directory containing source icon files and manifest.
  #[clap(default_value = "./app-icon.png")]
  input: PathBuf,
  /// Output directory.
  /// Default: 'icons' directory next to the tauri.conf.json file.
  #[clap(short, long)]
  output: Option<PathBuf>,

  /// Custom PNG icon sizes to generate. When set, the default icons are not generated.
  #[clap(short, long, use_value_delimiter = true)]
  png: Option<Vec<u32>>,

  /// The background color of the iOS icon - string as defined in the W3C's CSS Color Module Level 4 <https://www.w3.org/TR/css-color-4/>.
  #[clap(long, default_value = "#fff")]
  ios_color: String,
}

#[allow(clippy::large_enum_variant)]
enum Source {
  Svg(resvg::usvg::Tree),
  DynamicImage(DynamicImage),
}

impl Source {
  fn width(&self) -> u32 {
    match self {
      Self::Svg(svg) => svg.size().width() as u32,
      Self::DynamicImage(i) => i.width(),
    }
  }

  fn height(&self) -> u32 {
    match self {
      Self::Svg(svg) => svg.size().height() as u32,
      Self::DynamicImage(i) => i.height(),
    }
  }

  fn resize_exact(&self, size: u32) -> Result<DynamicImage> {
    match self {
      Self::Svg(svg) => {
        let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
        let scale = size as f32 / svg.size().height();
        resvg::render(
          svg,
          tiny_skia::Transform::from_scale(scale, scale),
          &mut pixmap.as_mut(),
        );
        let img_buffer = ImageBuffer::from_raw(size, size, pixmap.take()).unwrap();
        Ok(DynamicImage::ImageRgba8(img_buffer))
      }
      Self::DynamicImage(i) => Ok(i.resize_exact(size, size, FilterType::Lanczos3)),
    }
  }
}

fn read_source(path: PathBuf) -> Result<Source> {
  if let Some(extension) = path.extension() {
    if extension == "svg" {
      let rtree = {
        let mut fontdb = usvg::fontdb::Database::new();
        fontdb.load_system_fonts();

        let opt = usvg::Options {
          // Get file's absolute directory.
          resources_dir: std::fs::canonicalize(&path)
            .ok()
            .and_then(|p| p.parent().map(|p| p.to_path_buf())),
          fontdb: Arc::new(fontdb),
          ..Default::default()
        };

        let svg_data = std::fs::read(&path).unwrap();
        usvg::Tree::from_data(&svg_data, &opt).unwrap()
      };

      Ok(Source::Svg(rtree))
    } else {
      Ok(Source::DynamicImage(DynamicImage::ImageRgba8(
        open(&path)
          .context(format!("Can't read and decode source image: {:?}", path))?
          .into_rgba8(),
      )))
    }
  } else {
    anyhow::bail!("Error loading image");
  }
}

fn parse_bg_color(bg_color_string: &String) -> Result<Rgba<u8>> {
  let bg_color = css_color::Srgb::from_str(bg_color_string)
    .map(|color| {
      Rgba([
        (color.red * 255.) as u8,
        (color.green * 255.) as u8,
        (color.blue * 255.) as u8,
        (color.alpha * 255.) as u8,
      ])
    })
    .map_err(|_| anyhow::anyhow!("failed to parse color {}", bg_color_string))?;

  Ok(bg_color)
}

pub fn command(options: Options) -> Result<()> {
  let input = options.input;
  let out_dir = options.output.unwrap_or_else(|| {
    crate::helpers::app_paths::resolve();
    tauri_dir().join("icons")
  });
  let png_icon_sizes = options.png.unwrap_or_default();

  create_dir_all(&out_dir).context("Can't create output directory")?;

  let manifest: Option<Manifest> = parse_manifest(&input)?;

  let bg_color_string = match manifest {
    Some(ref manifest) => &manifest
      .bg_color
      .as_ref()
      .unwrap_or(&options.ios_color)
      .clone(),
    None => &options.ios_color,
  };
  let bg_color = parse_bg_color(bg_color_string)?;

  let default_icon = match manifest {
    Some(ref manifest) => input.join(manifest.default.clone()),
    None => input.clone(),
  };

  let source = read_source(default_icon)?;

  if source.height() != source.width() {
    anyhow::bail!("Source image must be square");
  }

  if png_icon_sizes.is_empty() {
    appx(&source, &out_dir).context("Failed to generate appx icons")?;
    icns(&source, &out_dir).context("Failed to generate .icns file")?;
    ico(&source, &out_dir).context("Failed to generate .ico file")?;

    png(&source, &out_dir, bg_color).context("Failed to generate png icons")?;
    android(&input, manifest, bg_color_string, &out_dir)
      .context("Failed to generate android icons")?;
  } else {
    for target in png_icon_sizes
      .into_iter()
      .map(|size| {
        let name = format!("{size}x{size}.png");
        let out_path = out_dir.join(&name);
        PngEntry {
          name,
          out_path,
          size,
        }
      })
      .collect::<Vec<PngEntry>>()
    {
      log::info!(action = "PNG"; "Creating {}", target.name);
      resize_and_save_png(&source, target.size, &target.out_path, None)?;
    }
  }

  Ok(())
}

fn parse_manifest(input: &Path) -> Result<Option<Manifest>> {
  if input.is_dir() {
    let manifest_path = input.join("manifest.json");
    if manifest_path.exists() {
      let manifest: Manifest = serde_json::from_str(
        &std::fs::read_to_string(&manifest_path)
          .expect("Cannot read manifest.json file in source directory"),
      )?;
      log::info!("Read manifest file from {}", manifest_path.display());
      return Ok(Some(manifest));
    }
  }
  Ok(None)
}

fn appx(source: &Source, out_dir: &Path) -> Result<()> {
  log::info!(action = "Appx"; "Creating StoreLogo.png");
  resize_and_save_png(source, 50, &out_dir.join("StoreLogo.png"), None)?;

  for size in [30, 44, 71, 89, 107, 142, 150, 284, 310] {
    let file_name = format!("Square{size}x{size}Logo.png");
    log::info!(action = "Appx"; "Creating {}", file_name);

    resize_and_save_png(source, size, &out_dir.join(&file_name), None)?;
  }

  Ok(())
}

// Main target: macOS
fn icns(source: &Source, out_dir: &Path) -> Result<()> {
  log::info!(action = "ICNS"; "Creating icon.icns");
  let entries: HashMap<String, IcnsEntry> =
    serde_json::from_slice(include_bytes!("helpers/icns.json")).unwrap();

  let mut family = IconFamily::new();

  for (name, entry) in entries {
    let size = entry.size;
    let mut buf = Vec::new();

    let image = source.resize_exact(size)?;

    write_png(image.as_bytes(), &mut buf, size)?;

    let image = icns::Image::read_png(&buf[..])?;

    family
      .add_icon_with_type(
        &image,
        IconType::from_ostype(entry.ostype.parse().unwrap()).unwrap(),
      )
      .with_context(|| format!("Can't add {name} to Icns Family"))?;
  }

  let mut out_file = BufWriter::new(File::create(out_dir.join("icon.icns"))?);
  family.write(&mut out_file)?;
  out_file.flush()?;

  Ok(())
}

// Generate .ico file with layers for the most common sizes.
// Main target: Windows
fn ico(source: &Source, out_dir: &Path) -> Result<()> {
  log::info!(action = "ICO"; "Creating icon.ico");
  let mut frames = Vec::new();

  for size in [32, 16, 24, 48, 64, 256] {
    let image = source.resize_exact(size)?;

    // Only the 256px layer can be compressed according to the ico specs.
    if size == 256 {
      let mut buf = Vec::new();

      write_png(image.as_bytes(), &mut buf, size)?;

      frames.push(IcoFrame::with_encoded(
        buf,
        size,
        size,
        ExtendedColorType::Rgba8,
      )?)
    } else {
      frames.push(IcoFrame::as_png(
        image.as_bytes(),
        size,
        size,
        ExtendedColorType::Rgba8,
      )?);
    }
  }

  let mut out_file = BufWriter::new(File::create(out_dir.join("icon.ico"))?);
  let encoder = IcoEncoder::new(&mut out_file);
  encoder.encode_images(&frames)?;
  out_file.flush()?;

  Ok(())
}

fn android(
  input: &Path,
  manifest: Option<Manifest>,
  bg_color: &String,
  out_dir: &Path,
) -> Result<()> {
  fn android_entries(out_dir: &Path) -> Result<AndroidEntries> {
    struct AndroidEntry {
      name: &'static str,
      size: u32,
      foreground_size: u32,
    }

    let targets = vec![
      AndroidEntry {
        name: "hdpi",
        size: 49,
        foreground_size: 162,
      },
      AndroidEntry {
        name: "mdpi",
        size: 48,
        foreground_size: 108,
      },
      AndroidEntry {
        name: "xhdpi",
        size: 96,
        foreground_size: 216,
      },
      AndroidEntry {
        name: "xxhdpi",
        size: 144,
        foreground_size: 324,
      },
      AndroidEntry {
        name: "xxxhdpi",
        size: 192,
        foreground_size: 432,
      },
    ];
    let mut fg_entries = Vec::new();
    let mut bg_entries = Vec::new();
    let mut monochrome_entries = Vec::new();

    for target in targets {
      let folder_name = format!("mipmap-{}", target.name);
      let out_folder = out_dir.join(&folder_name);

      create_dir_all(&out_folder).context("Can't create Android mipmap output directory")?;

      fg_entries.push(PngEntry {
        name: format!("{}/{}", folder_name, "ic_launcher_foreground.png"),
        out_path: out_folder.join("ic_launcher_foreground.png"),
        size: target.foreground_size,
      });
      fg_entries.push(PngEntry {
        name: format!("{}/{}", folder_name, "ic_launcher_round.png"),
        out_path: out_folder.join("ic_launcher_round.png"),
        size: target.size,
      });
      fg_entries.push(PngEntry {
        name: format!("{}/{}", folder_name, "ic_launcher.png"),
        out_path: out_folder.join("ic_launcher.png"),
        size: target.size,
      });

      bg_entries.push(PngEntry {
        name: format!("{}/{}", folder_name, "ic_launcher_background.png"),
        out_path: out_folder.join("ic_launcher_background.png"),
        size: target.foreground_size,
      });

      monochrome_entries.push(PngEntry {
        name: format!("{}/{}", folder_name, "ic_launcher_monochrome.png"),
        out_path: out_folder.join("ic_launcher_monochrome.png"),
        size: target.foreground_size,
      });
    }

    Ok(AndroidEntries {
      foreground: fg_entries,
      background: bg_entries,
      monochrome: monochrome_entries,
    })
  }
  fn create_color_file(out_dir: &Path, color: &String) -> Result<()> {
    let values_folder = out_dir.join("values");
    create_dir_all(&values_folder).context("Can't create Android values output directory")?;
    let mut color_file = File::create(values_folder.join("ic_launcher_background.xml"))?;
    color_file.write_all(
      format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
  <color name="ic_launcher_background">{}</color>
</resources>"#,
        color
      )
      .as_bytes(),
    )?;
    Ok(())
  }

  let android_out = out_dir
    .parent()
    .unwrap()
    .join("gen/android/app/src/main/res/");
  let out = if android_out.exists() {
    android_out
  } else {
    let out = out_dir.join("android");
    create_dir_all(&out).context("Can't create Android output directory")?;
    out
  };
  let entries = android_entries(&out)?;

  let foregrond_path = match manifest {
    Some(ref manifest) => input.join(manifest.android_fg.as_ref().unwrap_or(&manifest.default)),
    None => input.to_path_buf(),
  };

  let fg = read_source(foregrond_path)?;

  for entry in entries.foreground {
    log::info!(action = "Android"; "Creating {}", entry.name);
    resize_and_save_png(&fg, entry.size, &entry.out_path, None)?;
  }

  let mut has_bg_image = false;
  let mut has_monochrome_image = false;
  if let Some(ref manifest) = manifest {
    if let Some(ref background_path) = manifest.android_bg {
      has_bg_image = true;
      let bg = read_source(input.join(background_path))?;
      for entry in entries.background {
        log::info!(action = "Android"; "Creating {}", entry.name);
        resize_and_save_png(&bg, entry.size, &entry.out_path, None)?;
      }
    }
    if let Some(ref monochrome_path) = manifest.android_monochrome {
      has_monochrome_image = true;
      let mc = read_source(input.join(monochrome_path))?;
      for entry in entries.monochrome {
        log::info!(action = "Android"; "Creating {}", entry.name);
        resize_and_save_png(&mc, entry.size, &entry.out_path, None)?;
      }
    }
  }

  let mut launcher_content = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <foreground android:drawable="@mipmap/ic_launcher_foreground"/>"#
    .to_owned();

  if has_bg_image {
    launcher_content
      .push_str("\n  <background android:drawable=\"@mipmap/ic_launcher_background\"/>");
  } else {
    create_color_file(&out, bg_color)?;
    launcher_content
      .push_str("\n  <background android:drawable=\"@color/ic_launcher_background\"/>");
  }
  if has_monochrome_image {
    launcher_content
      .push_str("\n  <monochrome android:drawable=\"@mipmap/ic_launcher_monochrome\"/>");
  }
  launcher_content.push_str("\n</adaptive-icon>");

  let any_dpi_folder = out.join("mipmap-anydpi-v26");
  create_dir_all(&any_dpi_folder)
    .context("Can't create Android mipmap-anydpi-v26 output directory")?;
  let mut launcher_file = File::create(any_dpi_folder.join("ic_launcher.xml"))?;
  launcher_file.write_all(launcher_content.as_bytes())?;

  Ok(())
}

// Generate .png files in 32x32, 64x64, 128x128, 256x256, 512x512 (icon.png)
// Main target: Linux
fn png(source: &Source, out_dir: &Path, ios_color: Rgba<u8>) -> Result<()> {
  fn desktop_entries(out_dir: &Path) -> Vec<PngEntry> {
    let mut entries = Vec::new();

    for size in [32, 64, 128, 256, 512] {
      let file_name = match size {
        256 => "128x128@2x.png".to_string(),
        512 => "icon.png".to_string(),
        _ => format!("{size}x{size}.png"),
      };

      entries.push(PngEntry {
        out_path: out_dir.join(&file_name),
        name: file_name,
        size,
      });
    }

    entries
  }

  fn ios_entries(out_dir: &Path) -> Result<Vec<PngEntry>> {
    struct IosEntry {
      size: f32,
      multipliers: Vec<u8>,
      has_extra: bool,
    }

    let mut entries = Vec::new();

    let targets = vec![
      IosEntry {
        size: 20.,
        multipliers: vec![1, 2, 3],
        has_extra: true,
      },
      IosEntry {
        size: 29.,
        multipliers: vec![1, 2, 3],
        has_extra: true,
      },
      IosEntry {
        size: 40.,
        multipliers: vec![1, 2, 3],
        has_extra: true,
      },
      IosEntry {
        size: 60.,
        multipliers: vec![2, 3],
        has_extra: false,
      },
      IosEntry {
        size: 76.,
        multipliers: vec![1, 2],
        has_extra: false,
      },
      IosEntry {
        size: 83.5,
        multipliers: vec![2],
        has_extra: false,
      },
      IosEntry {
        size: 512.,
        multipliers: vec![2],
        has_extra: false,
      },
    ];

    for target in targets {
      let size_str = if target.size == 512. {
        "512".to_string()
      } else {
        format!("{size}x{size}", size = target.size)
      };
      if target.has_extra {
        let name = format!("AppIcon-{size_str}@2x-1.png");
        entries.push(PngEntry {
          out_path: out_dir.join(&name),
          name,
          size: (target.size * 2.) as u32,
        });
      }
      for multiplier in target.multipliers {
        let name = format!("AppIcon-{size_str}@{multiplier}x.png");
        entries.push(PngEntry {
          out_path: out_dir.join(&name),
          name,
          size: (target.size * multiplier as f32) as u32,
        });
      }
    }

    Ok(entries)
  }

  let entries = desktop_entries(out_dir);

  let ios_out = out_dir
    .parent()
    .unwrap()
    .join("gen/apple/Assets.xcassets/AppIcon.appiconset");
  let out = if ios_out.exists() {
    ios_out
  } else {
    let out = out_dir.join("ios");
    create_dir_all(&out).context("Can't create iOS output directory")?;
    out
  };

  for entry in entries {
    log::info!(action = "PNG"; "Creating {}", entry.name);
    resize_and_save_png(source, entry.size, &entry.out_path, None)?;
  }

  for entry in ios_entries(&out)? {
    log::info!(action = "iOS"; "Creating {}", entry.name);
    resize_and_save_png(source, entry.size, &entry.out_path, Some(ios_color))?;
  }

  Ok(())
}

// Resize image and save it to disk.
fn resize_and_save_png(
  source: &Source,
  size: u32,
  file_path: &Path,
  bg_color: Option<Rgba<u8>>,
) -> Result<()> {
  let mut image = source.resize_exact(size)?;

  if let Some(bg_color) = bg_color {
    let mut bg_img = ImageBuffer::from_fn(size, size, |_, _| bg_color);
    image::imageops::overlay(&mut bg_img, &image, 0, 0);
    image = bg_img.into();
  }

  let mut out_file = BufWriter::new(File::create(file_path)?);
  write_png(image.as_bytes(), &mut out_file, size)?;
  Ok(out_file.flush()?)
}

// Encode image data as png with compression.
fn write_png<W: Write>(image_data: &[u8], w: W, size: u32) -> Result<()> {
  let encoder = PngEncoder::new_with_quality(w, CompressionType::Best, PngFilterType::Adaptive);
  encoder.write_image(image_data, size, size, ExtendedColorType::Rgba8)?;
  Ok(())
}
