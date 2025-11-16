// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::{Context, Error, ErrorExt},
  helpers::app_paths::tauri_dir,
  Result,
};

use std::{
  borrow::Cow,
  collections::HashMap,
  fs::{create_dir_all, File},
  io::{BufWriter, Write},
  path::{Path, PathBuf},
  str::FromStr,
  sync::Arc,
};

use clap::Parser;
use icns::{IconFamily, IconType};
use image::{
  codecs::{
    ico::{IcoEncoder, IcoFrame},
    png::{CompressionType, FilterType as PngFilterType, PngEncoder},
  },
  imageops::FilterType,
  open, DynamicImage, ExtendedColorType, GenericImageView, ImageBuffer, ImageEncoder, Pixel, Rgba,
};
use rayon::iter::ParallelIterator;
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

enum AndroidIconKind {
  Regular,
  Rounded,
}

struct AndroidEntries {
  icon: Vec<(PngEntry, AndroidIconKind)>,
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
  android_fg_scale: Option<f32>,
}

#[derive(Debug, Parser)]
#[clap(about = "Generate various icons for all major platforms")]
pub struct Options {
  /// Path to the source icon (squared PNG or SVG file with transparency) or a manifest file.
  ///
  /// The manifest file is a JSON file with the following structure:
  /// {
  ///   "default": "app-icon.png",
  ///   "bg_color": "#fff",
  ///   "android_bg": "app-icon-bg.png",
  ///   "android_fg": "app-icon-fg.png",
  ///   "android_fg_scale": 85,
  ///   "android_monochrome": "app-icon-monochrome.png"
  /// }
  ///
  /// All file paths defined in the manifest JSON are relative to the manifest file path.
  ///
  /// Only the `default` manifest property is required.
  ///
  /// The `bg_color` manifest value overwrites the `--ios-color` option if set.
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

#[derive(Clone)]
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

  fn resize_exact(&self, size: u32) -> DynamicImage {
    match self {
      Self::Svg(svg) => {
        let mut pixmap = tiny_skia::Pixmap::new(size, size).unwrap();
        let scale = size as f32 / svg.size().height();
        resvg::render(
          svg,
          tiny_skia::Transform::from_scale(scale, scale),
          &mut pixmap.as_mut(),
        );
        // Switch to use `Pixmap::take_demultiplied` in the future when it's published
        // https://github.com/linebender/tiny-skia/blob/624257c0feb394bf6c4d0d688f8ea8030aae320f/src/pixmap.rs#L266
        let img_buffer = ImageBuffer::from_par_fn(size, size, |x, y| {
          let pixel = pixmap.pixel(x, y).unwrap().demultiply();
          Rgba([pixel.red(), pixel.green(), pixel.blue(), pixel.alpha()])
        });
        DynamicImage::ImageRgba8(img_buffer)
      }
      Self::DynamicImage(image) => {
        // image.resize_exact(size, size, FilterType::Lanczos3)
        resize_image(image, size, size)
      }
    }
  }
}

// `image` does not use premultiplied alpha in resize, so we do it manually here,
// see https://github.com/image-rs/image/issues/1655
fn resize_image(image: &DynamicImage, new_width: u32, new_height: u32) -> DynamicImage {
  // Premultiply alpha
  let premultiplied_image = ImageBuffer::from_par_fn(image.width(), image.height(), |x, y| {
    let mut pixel = image.get_pixel(x, y);
    let alpha = pixel.0[3] as f32 / u8::MAX as f32;
    pixel.apply_without_alpha(|channel_value| (channel_value as f32 * alpha) as u8);
    pixel
  });

  let mut resized = image::imageops::resize(
    &premultiplied_image,
    new_width,
    new_height,
    FilterType::Lanczos3,
  );

  // Demultiply alpha
  resized.par_pixels_mut().for_each(|pixel| {
    let alpha = pixel.0[3] as f32 / u8::MAX as f32;
    pixel.apply_without_alpha(|channel_value| (channel_value as f32 / alpha) as u8);
  });

  DynamicImage::ImageRgba8(resized)
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

        let svg_data = std::fs::read(&path).fs_context("Failed to read source icon", &path)?;
        usvg::Tree::from_data(&svg_data, &opt).unwrap()
      };

      Ok(Source::Svg(rtree))
    } else {
      Ok(Source::DynamicImage(DynamicImage::ImageRgba8(
        open(&path)
          .context(format!(
            "failed to read and decode source image {}",
            path.display()
          ))?
          .into_rgba8(),
      )))
    }
  } else {
    crate::error::bail!("Error loading image");
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
    .map_err(|_e| {
      Error::Context(
        format!("failed to parse color {bg_color_string}"),
        "invalid RGBA color".into(),
      )
    })?;

  Ok(bg_color)
}

pub fn command(options: Options) -> Result<()> {
  let input = options.input;
  let out_dir = options.output.unwrap_or_else(|| {
    crate::helpers::app_paths::resolve();
    tauri_dir().join("icons")
  });
  let png_icon_sizes = options.png.unwrap_or_default();

  create_dir_all(&out_dir).fs_context("Can't create output directory", &out_dir)?;

  let manifest = if input.extension().is_some_and(|ext| ext == "json") {
    parse_manifest(&input).map(Some)?
  } else {
    None
  };

  let bg_color_string = match manifest {
    Some(ref manifest) => manifest
      .bg_color
      .as_ref()
      .unwrap_or(&options.ios_color)
      .clone(),
    None => options.ios_color,
  };
  let bg_color = parse_bg_color(&bg_color_string)?;

  let default_icon = match manifest {
    Some(ref manifest) => input.parent().unwrap().join(manifest.default.clone()),
    None => input.clone(),
  };

  let source = read_source(default_icon)?;

  if source.height() != source.width() {
    crate::error::bail!("Source image must be square");
  }

  if png_icon_sizes.is_empty() {
    appx(&source, &out_dir).context("Failed to generate appx icons")?;
    icns(&source, &out_dir).context("Failed to generate .icns file")?;
    ico(&source, &out_dir).context("Failed to generate .ico file")?;

    png(&source, &out_dir, bg_color).context("Failed to generate png icons")?;
    android(&source, &input, manifest, &bg_color_string, &out_dir)
      .context("Failed to generate android icons")?;
  } else {
    for target in png_icon_sizes.into_iter().map(|size| {
      let name = format!("{size}x{size}.png");
      let out_path = out_dir.join(&name);
      PngEntry {
        name,
        out_path,
        size,
      }
    }) {
      log::info!(action = "PNG"; "Creating {}", target.name);
      resize_and_save_png(&source, target.size, &target.out_path, None, None)?;
    }
  }

  Ok(())
}

fn parse_manifest(manifest_path: &Path) -> Result<Manifest> {
  let manifest: Manifest = serde_json::from_str(
    &std::fs::read_to_string(manifest_path)
      .fs_context("cannot read manifest file", manifest_path)?,
  )
  .context(format!(
    "failed to parse manifest file {}",
    manifest_path.display()
  ))?;
  log::debug!("Read manifest file from {}", manifest_path.display());
  Ok(manifest)
}

fn appx(source: &Source, out_dir: &Path) -> Result<()> {
  log::info!(action = "Appx"; "Creating StoreLogo.png");
  resize_and_save_png(source, 50, &out_dir.join("StoreLogo.png"), None, None)?;

  for size in [30, 44, 71, 89, 107, 142, 150, 284, 310] {
    let file_name = format!("Square{size}x{size}Logo.png");
    log::info!(action = "Appx"; "Creating {}", file_name);

    resize_and_save_png(source, size, &out_dir.join(&file_name), None, None)?;
  }

  Ok(())
}

// Main target: macOS
fn icns(source: &Source, out_dir: &Path) -> Result<()> {
  log::info!(action = "ICNS"; "Creating icon.icns");
  let entries: HashMap<String, IcnsEntry> =
    serde_json::from_slice(include_bytes!("helpers/icns.json")).unwrap();

  let mut family = IconFamily::new();

  for (_name, entry) in entries {
    let size = entry.size;
    let mut buf = Vec::new();

    let image = source.resize_exact(size);

    write_png(image.as_bytes(), &mut buf, size).context("failed to write output file")?;

    let image = icns::Image::read_png(&buf[..]).context("failed to read output file")?;

    family
      .add_icon_with_type(
        &image,
        IconType::from_ostype(entry.ostype.parse().unwrap()).unwrap(),
      )
      .context("failed to add icon to Icns Family")?;
  }

  let icns_path = out_dir.join("icon.icns");
  let mut out_file = BufWriter::new(
    File::create(&icns_path).fs_context("failed to create output file", &icns_path)?,
  );
  family
    .write(&mut out_file)
    .fs_context("failed to write output file", &icns_path)?;
  out_file
    .flush()
    .fs_context("failed to flush output file", &icns_path)?;

  Ok(())
}

// Generate .ico file with layers for the most common sizes.
// Main target: Windows
fn ico(source: &Source, out_dir: &Path) -> Result<()> {
  log::info!(action = "ICO"; "Creating icon.ico");
  let mut frames = Vec::new();

  for size in [32, 16, 24, 48, 64, 256] {
    let image = source.resize_exact(size);

    // Only the 256px layer can be compressed according to the ico specs.
    if size == 256 {
      let mut buf = Vec::new();

      write_png(image.as_bytes(), &mut buf, size).context("failed to write output file")?;

      frames.push(
        IcoFrame::with_encoded(buf, size, size, ExtendedColorType::Rgba8)
          .context("failed to create ico frame")?,
      );
    } else {
      frames.push(
        IcoFrame::as_png(image.as_bytes(), size, size, ExtendedColorType::Rgba8)
          .context("failed to create PNG frame")?,
      );
    }
  }

  let ico_path = out_dir.join("icon.ico");
  let mut out_file =
    BufWriter::new(File::create(&ico_path).fs_context("failed to create output file", &ico_path)?);
  let encoder = IcoEncoder::new(&mut out_file);
  encoder
    .encode_images(&frames)
    .context("failed to encode images")?;
  out_file
    .flush()
    .fs_context("failed to flush output file", &ico_path)?;

  Ok(())
}

fn android(
  source: &Source,
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
    let mut icon_entries = Vec::new();
    let mut fg_entries = Vec::new();
    let mut bg_entries = Vec::new();
    let mut monochrome_entries = Vec::new();

    for target in targets {
      let folder_name = format!("mipmap-{}", target.name);
      let out_folder = out_dir.join(&folder_name);

      create_dir_all(&out_folder).fs_context(
        "failed to create Android mipmap output directory",
        &out_folder,
      )?;

      fg_entries.push(PngEntry {
        name: format!("{}/{}", folder_name, "ic_launcher_foreground.png"),
        out_path: out_folder.join("ic_launcher_foreground.png"),
        size: target.foreground_size,
      });
      icon_entries.push((
        PngEntry {
          name: format!("{}/{}", folder_name, "ic_launcher_round.png"),
          out_path: out_folder.join("ic_launcher_round.png"),
          size: target.size,
        },
        AndroidIconKind::Rounded,
      ));
      icon_entries.push((
        PngEntry {
          name: format!("{}/{}", folder_name, "ic_launcher.png"),
          out_path: out_folder.join("ic_launcher.png"),
          size: target.size,
        },
        AndroidIconKind::Regular,
      ));

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
      icon: icon_entries,
      foreground: fg_entries,
      background: bg_entries,
      monochrome: monochrome_entries,
    })
  }
  fn create_color_file(out_dir: &Path, color: &String) -> Result<()> {
    let values_folder = out_dir.join("values");
    create_dir_all(&values_folder).fs_context(
      "Can't create Android values output directory",
      &values_folder,
    )?;
    let launcher_background_xml_path = values_folder.join("ic_launcher_background.xml");
    let mut color_file = File::create(&launcher_background_xml_path).fs_context(
      "failed to create Android color file",
      &launcher_background_xml_path,
    )?;
    color_file
      .write_all(
        format!(
          r#"<?xml version="1.0" encoding="utf-8"?>
<resources>
  <color name="ic_launcher_background">{color}</color>
</resources>"#,
        )
        .as_bytes(),
      )
      .fs_context(
        "failed to write Android color file",
        &launcher_background_xml_path,
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
    create_dir_all(&out).fs_context("Can't create Android output directory", &out)?;
    out
  };
  let entries = android_entries(&out)?;

  let fg_source = match manifest {
    Some(ref manifest) => {
      Some(read_source(input.parent().unwrap().join(
        manifest.android_fg.as_ref().unwrap_or(&manifest.default),
      ))?)
    }
    None => None,
  };

  for entry in entries.foreground {
    log::info!(action = "Android"; "Creating {}", entry.name);
    resize_and_save_png(
      fg_source.as_ref().unwrap_or(source),
      entry.size,
      &entry.out_path,
      None,
      None,
    )?;
  }

  let mut bg_source = None;
  let mut has_monochrome_image = false;
  if let Some(ref manifest) = manifest {
    if let Some(ref background_path) = manifest.android_bg {
      let bg = read_source(input.parent().unwrap().join(background_path))?;
      for entry in entries.background {
        log::info!(action = "Android"; "Creating {}", entry.name);
        resize_and_save_png(&bg, entry.size, &entry.out_path, None, None)?;
      }
      bg_source.replace(bg);
    }
    if let Some(ref monochrome_path) = manifest.android_monochrome {
      has_monochrome_image = true;
      let mc = read_source(input.parent().unwrap().join(monochrome_path))?;
      for entry in entries.monochrome {
        log::info!(action = "Android"; "Creating {}", entry.name);
        resize_and_save_png(&mc, entry.size, &entry.out_path, None, None)?;
      }
    }
  }

  for (entry, kind) in entries.icon {
    log::info!(action = "Android"; "Creating {}", entry.name);

    let (margin, radius) = match kind {
      AndroidIconKind::Regular => {
        let radius = ((entry.size as f32) * 0.0833).round() as u32;
        (radius, radius)
      }
      AndroidIconKind::Rounded => {
        let margin = ((entry.size as f32) * 0.04).round() as u32;
        let radius = ((entry.size as f32) * 0.5).round() as u32;
        (margin, radius)
      }
    };

    let image = if let (Some(bg_source), Some(fg_source)) = (bg_source.as_ref(), fg_source.as_ref())
    {
      resize_png(
        fg_source,
        entry.size,
        Some(Background::Image(bg_source)),
        manifest
          .as_ref()
          .and_then(|manifest| manifest.android_fg_scale),
      )?
    } else {
      resize_png(source, entry.size, None, None)?
    };

    let image = apply_round_mask(&image, entry.size, margin, radius);

    let mut out_file = BufWriter::new(
      File::create(&entry.out_path).fs_context("failed to create output file", &entry.out_path)?,
    );
    write_png(image.as_bytes(), &mut out_file, entry.size)
      .context("failed to write output file")?;
    out_file
      .flush()
      .fs_context("failed to flush output file", &entry.out_path)?;
  }

  let mut launcher_content = r#"<?xml version="1.0" encoding="utf-8"?>
<adaptive-icon xmlns:android="http://schemas.android.com/apk/res/android">
  <foreground android:drawable="@mipmap/ic_launcher_foreground"/>"#
    .to_owned();

  if bg_source.is_some() {
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
  create_dir_all(&any_dpi_folder).fs_context(
    "Can't create Android mipmap-anydpi-v26 output directory",
    &any_dpi_folder,
  )?;

  let launcher_xml_path = any_dpi_folder.join("ic_launcher.xml");
  let mut launcher_file = File::create(&launcher_xml_path)
    .fs_context("failed to create Android launcher file", &launcher_xml_path)?;
  launcher_file
    .write_all(launcher_content.as_bytes())
    .fs_context("failed to write Android launcher file", &launcher_xml_path)?;

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
    create_dir_all(&out).fs_context("failed to create iOS output directory", &out)?;
    out
  };

  for entry in entries {
    log::info!(action = "PNG"; "Creating {}", entry.name);
    resize_and_save_png(source, entry.size, &entry.out_path, None, None)?;
  }

  for entry in ios_entries(&out)? {
    log::info!(action = "iOS"; "Creating {}", entry.name);
    resize_and_save_png(
      source,
      entry.size,
      &entry.out_path,
      Some(Background::Color(ios_color)),
      None,
    )?;
  }

  Ok(())
}

enum Background<'a> {
  Color(Rgba<u8>),
  Image(&'a Source),
}

// Resize image.
fn resize_png(
  source: &Source,
  size: u32,
  bg: Option<Background>,
  scale_percent: Option<f32>,
) -> Result<DynamicImage> {
  let mut image = source.resize_exact(size);

  match bg {
    Some(Background::Color(bg_color)) => {
      let mut bg_img = ImageBuffer::from_fn(size, size, |_, _| bg_color);

      let fg = scale_percent
        .map(|scale| resize_asset(&image, size, scale))
        .unwrap_or(image);

      image::imageops::overlay(&mut bg_img, &fg, 0, 0);
      image = bg_img.into();
    }
    Some(Background::Image(bg_source)) => {
      let mut bg = bg_source.resize_exact(size);

      let fg = scale_percent
        .map(|scale| resize_asset(&image, size, scale))
        .unwrap_or(image);

      image::imageops::overlay(&mut bg, &fg, 0, 0);
      image = bg;
    }
    None => {}
  }

  Ok(image)
}

// Resize image and save it to disk.
fn resize_and_save_png(
  source: &Source,
  size: u32,
  file_path: &Path,
  bg: Option<Background>,
  scale_percent: Option<f32>,
) -> Result<()> {
  let image = resize_png(source, size, bg, scale_percent)?;
  let mut out_file =
    BufWriter::new(File::create(file_path).fs_context("failed to create output file", file_path)?);
  write_png(image.as_bytes(), &mut out_file, size).context("failed to write output file")?;
  out_file
    .flush()
    .fs_context("failed to save output file", file_path)
}

// Encode image data as png with compression.
fn write_png<W: Write>(image_data: &[u8], w: W, size: u32) -> image::ImageResult<()> {
  let encoder = PngEncoder::new_with_quality(w, CompressionType::Best, PngFilterType::Adaptive);
  encoder.write_image(image_data, size, size, ExtendedColorType::Rgba8)?;
  Ok(())
}

// finds the bounding box of non-transparent pixels in an RGBA image.
fn content_bounds(img: &DynamicImage) -> Option<(u32, u32, u32, u32)> {
  let rgba = img.to_rgba8();
  let (width, height) = img.dimensions();

  let mut min_x = width;
  let mut min_y = height;
  let mut max_x = 0;
  let mut max_y = 0;
  let mut found = false;

  for y in 0..height {
    for x in 0..width {
      let a = rgba.get_pixel(x, y)[3];
      if a > 0 {
        found = true;
        if x < min_x {
          min_x = x;
        }
        if y < min_y {
          min_y = y;
        }
        if x > max_x {
          max_x = x;
        }
        if y > max_y {
          max_y = y;
        }
      }
    }
  }

  if found {
    Some((min_x, min_y, max_x - min_x + 1, max_y - min_y + 1))
  } else {
    None
  }
}

fn resize_asset(img: &DynamicImage, target_size: u32, scale_percent: f32) -> DynamicImage {
  let cropped = if let Some((x, y, cw, ch)) = content_bounds(img) {
    // TODO: Use `&` here instead when we raise MSRV to above 1.79
    Cow::Owned(img.crop_imm(x, y, cw, ch))
  } else {
    Cow::Borrowed(img)
  };

  let (cw, ch) = cropped.dimensions();
  let max_dim = cw.max(ch) as f32;
  let scale = (target_size as f32 * (scale_percent / 100.0)) / max_dim;

  let new_w = (cw as f32 * scale).round() as u32;
  let new_h = (ch as f32 * scale).round() as u32;

  let resized = resize_image(&cropped, new_w, new_h);

  // Place on transparent square canvas
  let mut canvas = ImageBuffer::from_pixel(target_size, target_size, Rgba([0, 0, 0, 0]));
  let offset_x = if new_w > target_size {
    // Image wider than canvas → start at negative offset
    -((new_w - target_size) as i32 / 2)
  } else {
    (target_size - new_w) as i32 / 2
  };

  let offset_y = if new_h > target_size {
    -((new_h - target_size) as i32 / 2)
  } else {
    (target_size - new_h) as i32 / 2
  };

  image::imageops::overlay(&mut canvas, &resized, offset_x.into(), offset_y.into());

  DynamicImage::ImageRgba8(canvas)
}

fn apply_round_mask(
  img: &DynamicImage,
  target_size: u32,
  margin: u32,
  radius: u32,
) -> DynamicImage {
  // Clamp radius to half of inner size
  let inner_size = target_size.saturating_sub(2 * margin);
  let radius = radius.min(inner_size / 2);

  // Resize inner image to fit inside margins
  let resized = img.resize_exact(inner_size, inner_size, image::imageops::Lanczos3);

  // Prepare output canvas
  let mut out = ImageBuffer::from_pixel(target_size, target_size, Rgba([0, 0, 0, 0]));

  // Draw the resized image at (margin, margin)
  image::imageops::overlay(&mut out, &resized, margin as i64, margin as i64);

  // Apply rounded corners
  for y in 0..target_size {
    for x in 0..target_size {
      let inside = if x >= margin + radius
        && x < target_size - margin - radius
        && y >= margin + radius
        && y < target_size - margin - radius
      {
        true // inside central rectangle
      } else {
        // Determine corner centers
        let (cx, cy) = if x < margin + radius && y < margin + radius {
          (margin + radius, margin + radius) // top-left
        } else if x >= target_size - margin - radius && y < margin + radius {
          (target_size - margin - radius, margin + radius) // top-right
        } else if x < margin + radius && y >= target_size - margin - radius {
          (margin + radius, target_size - margin - radius) // bottom-left
        } else if x >= target_size - margin - radius && y >= target_size - margin - radius {
          (target_size - margin - radius, target_size - margin - radius) // bottom-right
        } else {
          continue; // edges that are not corners are inside
        };
        let dx = x as i32 - cx as i32;
        let dy = y as i32 - cy as i32;
        dx * dx + dy * dy <= (radius as i32 * radius as i32)
      };

      if !inside {
        out.put_pixel(x, y, Rgba([0, 0, 0, 0]));
      }
    }
  }

  DynamicImage::ImageRgba8(out)
}
