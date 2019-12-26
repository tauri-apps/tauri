// An OSX package is laid out like:
//
// foobar.app    # Actually a directory
//     Contents      # A further subdirectory
//         Info.plist     # An xml file containing the app's metadata
//         MacOS          # A directory to hold executable binary files
//             foobar          # The main binary executable of the app
//             foobar_helper   # A helper application, possibly provitidng a CLI
//         Resources      # Data files such as images, sounds, translations and nib files
//             en.lproj        # Folder containing english translation strings/data
//         Frameworks     # A directory containing private frameworks (shared libraries)
//         ...            # Any other optional files the developer wants to place here
//
// See https://developer.apple.com/go/?id=bundle-structure for a full
// explanation.
//
// Currently, cargo-bundle does not support Frameworks, nor does it support placing arbitrary
// files into the `Contents` directory of the bundle.

use super::common;
use crate::{ResultExt, Settings};
use chrono;
use dirs;
use icns;
use image::{self, GenericImageView};
use std::cmp::min;
use std::ffi::OsStr;
use std::fs::{self, File};
use std::io::prelude::*;
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};

pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let app_bundle_name = format!("{}.app", settings.bundle_name());
  common::print_bundling(&app_bundle_name)?;
  let app_bundle_path = settings
    .project_out_directory()
    .join("bundle/osx")
    .join(&app_bundle_name);
  if app_bundle_path.exists() {
    fs::remove_dir_all(&app_bundle_path)
      .chain_err(|| format!("Failed to remove old {}", app_bundle_name))?;
  }
  let bundle_directory = app_bundle_path.join("Contents");
  fs::create_dir_all(&bundle_directory).chain_err(|| {
    format!(
      "Failed to create bundle directory at {:?}",
      bundle_directory
    )
  })?;

  let resources_dir = bundle_directory.join("Resources");

  let bundle_icon_file: Option<PathBuf> =
    { create_icns_file(&resources_dir, settings).chain_err(|| "Failed to create app icon")? };

  create_info_plist(&bundle_directory, bundle_icon_file, settings)
    .chain_err(|| "Failed to create Info.plist")?;

  copy_frameworks_to_bundle(&bundle_directory, settings)
    .chain_err(|| "Failed to bundle frameworks")?;

  for src in settings.resource_files() {
    let src = src?;
    let dest = resources_dir.join(common::resource_relpath(&src));
    common::copy_file(&src, &dest)
      .chain_err(|| format!("Failed to copy resource file {:?}", src))?;
  }

  copy_binary_to_bundle(&bundle_directory, settings)
    .chain_err(|| format!("Failed to copy binary from {:?}", settings.binary_path()))?;

  Ok(vec![app_bundle_path])
}

fn copy_binary_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  let dest_dir = bundle_directory.join("MacOS");
  common::copy_file(
    settings.binary_path(),
    &dest_dir.join(settings.binary_name()),
  )
}

fn create_info_plist(
  bundle_dir: &Path,
  bundle_icon_file: Option<PathBuf>,
  settings: &Settings,
) -> crate::Result<()> {
  let build_number = chrono::Utc::now().format("%Y%m%d.%H%M%S");
  let file = &mut common::create_file(&bundle_dir.join("Info.plist"))?;
  write!(
    file,
    "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
     <!DOCTYPE plist PUBLIC \"-//Apple Computer//DTD PLIST 1.0//EN\" \
     \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
     <plist version=\"1.0\">\n\
     <dict>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleDevelopmentRegion</key>\n  \
     <string>English</string>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleDisplayName</key>\n  <string>{}</string>\n",
    settings.bundle_name()
  )?;
  write!(
    file,
    "  <key>CFBundleExecutable</key>\n  <string>{}</string>\n",
    settings.binary_name()
  )?;
  if let Some(path) = bundle_icon_file {
    write!(
      file,
      "  <key>CFBundleIconFile</key>\n  <string>{}</string>\n",
      path.file_name().expect("No file name").to_string_lossy()
    )?;
  }
  write!(
    file,
    "  <key>CFBundleIdentifier</key>\n  <string>{}</string>\n",
    settings.bundle_identifier()
  )?;
  write!(
    file,
    "  <key>CFBundleInfoDictionaryVersion</key>\n  \
     <string>6.0</string>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleName</key>\n  <string>{}</string>\n",
    settings.bundle_name()
  )?;
  write!(
    file,
    "  <key>CFBundlePackageType</key>\n  <string>APPL</string>\n"
  )?;
  write!(
    file,
    "  <key>CFBundleShortVersionString</key>\n  <string>{}</string>\n",
    settings.version_string()
  )?;
  write!(
    file,
    "  <key>CFBundleVersion</key>\n  <string>{}</string>\n",
    build_number
  )?;
  write!(file, "  <key>CSResourcesFileMapped</key>\n  <true/>\n")?;
  if let Some(category) = settings.app_category() {
    write!(
      file,
      "  <key>LSApplicationCategoryType</key>\n  \
       <string>{}</string>\n",
      category.osx_application_category_type()
    )?;
  }
  if let Some(version) = settings.osx_minimum_system_version() {
    write!(
      file,
      "  <key>LSMinimumSystemVersion</key>\n  \
       <string>{}</string>\n",
      version
    )?;
  }
  write!(file, "  <key>LSRequiresCarbon</key>\n  <true/>\n")?;
  write!(file, "  <key>NSHighResolutionCapable</key>\n  <true/>\n")?;
  if let Some(copyright) = settings.copyright_string() {
    write!(
      file,
      "  <key>NSHumanReadableCopyright</key>\n  \
       <string>{}</string>\n",
      copyright
    )?;
  }
  write!(file, "</dict>\n</plist>\n")?;
  file.flush()?;
  Ok(())
}

fn copy_framework_from(dest_dir: &Path, framework: &str, src_dir: &Path) -> crate::Result<bool> {
  let src_name = format!("{}.framework", framework);
  let src_path = src_dir.join(&src_name);
  if src_path.exists() {
    common::copy_dir(&src_path, &dest_dir.join(&src_name))?;
    Ok(true)
  } else {
    Ok(false)
  }
}

fn copy_frameworks_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  let frameworks = settings.osx_frameworks();
  if frameworks.is_empty() {
    return Ok(());
  }
  let dest_dir = bundle_directory.join("Frameworks");
  fs::create_dir_all(&bundle_directory)
    .chain_err(|| format!("Failed to create Frameworks directory at {:?}", dest_dir))?;
  for framework in frameworks.iter() {
    if framework.ends_with(".framework") {
      let src_path = PathBuf::from(framework);
      let src_name = src_path.file_name().expect("Couldn't get file name");
      common::copy_dir(&src_path, &dest_dir.join(&src_name))?;
      continue;
    } else if framework.contains("/") {
      bail!(
        "Framework path should have .framework extension: {}",
        framework
      );
    }
    if let Some(home_dir) = dirs::home_dir() {
      if copy_framework_from(&dest_dir, framework, &home_dir.join("Library/Frameworks/"))? {
        continue;
      }
    }
    if copy_framework_from(&dest_dir, framework, &PathBuf::from("/Library/Frameworks/"))?
      || copy_framework_from(
        &dest_dir,
        framework,
        &PathBuf::from("/Network/Library/Frameworks/"),
      )?
    {
      continue;
    }
    bail!("Could not locate {}.framework", framework);
  }
  Ok(())
}

/// Given a list of icon files, try to produce an ICNS file in the resources
/// directory and return the path to it.  Returns `Ok(None)` if no usable icons
/// were provided.
fn create_icns_file(
  resources_dir: &PathBuf,
  settings: &Settings,
) -> crate::Result<Option<PathBuf>> {
  if settings.icon_files().count() == 0 {
    return Ok(None);
  }

  // If one of the icon files is already an ICNS file, just use that.
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;
    if icon_path.extension() == Some(OsStr::new("icns")) {
      let mut dest_path = resources_dir.to_path_buf();
      dest_path.push(icon_path.file_name().expect("Could not get filename"));
      common::copy_file(&icon_path, &dest_path)?;
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
    let density = if common::is_retina(&icon_path) { 2 } else { 1 };
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
    let icon = icon.resize_exact(next_size_down, next_size_down, image::Lanczos3);
    add_icon_to_family(icon, density, &mut family)?;
  }

  if !family.is_empty() {
    fs::create_dir_all(resources_dir)?;
    let mut dest_path = resources_dir.clone();
    dest_path.push(settings.bundle_name());
    dest_path.set_extension("icns");
    let icns_file = BufWriter::new(File::create(&dest_path)?);
    family.write(icns_file)?;
    return Ok(Some(dest_path));
  }

  bail!("No usable icon files found.");
}

/// Converts an image::DynamicImage into an icns::Image.
fn make_icns_image(img: image::DynamicImage) -> io::Result<icns::Image> {
  let pixel_format = match img.color() {
    image::ColorType::RGBA(8) => icns::PixelFormat::RGBA,
    image::ColorType::RGB(8) => icns::PixelFormat::RGB,
    image::ColorType::GrayA(8) => icns::PixelFormat::GrayAlpha,
    image::ColorType::Gray(8) => icns::PixelFormat::Gray,
    _ => {
      let msg = format!("unsupported ColorType: {:?}", img.color());
      return Err(io::Error::new(io::ErrorKind::InvalidData, msg));
    }
  };
  icns::Image::from_data(pixel_format, img.width(), img.height(), img.raw_pixels())
}
