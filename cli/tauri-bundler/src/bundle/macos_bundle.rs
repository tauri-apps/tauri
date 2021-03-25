// A macOS application bundle package is laid out like:
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
use crate::Settings;

use anyhow::Context;
use image::{self, GenericImageView};

use std::{
  cmp::min,
  ffi::OsStr,
  fs::{self, File},
  io::{self, prelude::*, BufWriter},
  path::{Path, PathBuf},
  process::{Command, Stdio},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the .app was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let package_base_name = format!(
    "{}_{}_{}",
    settings.main_binary_name(),
    settings.version_string(),
    match settings.binary_arch() {
      "x86_64" => "x64",
      other => other,
    }
  );
  let app_product_name = format!("{}.app", package_base_name);
  common::print_bundling(&app_product_name)?;
  let app_bundle_path = settings
    .project_out_directory()
    .join("bundle/macos")
    .join(&app_product_name);
  if app_bundle_path.exists() {
    fs::remove_dir_all(&app_bundle_path)
      .with_context(|| format!("Failed to remove old {}", app_product_name))?;
  }
  let bundle_directory = app_bundle_path.join("Contents");
  fs::create_dir_all(&bundle_directory).with_context(|| {
    format!(
      "Failed to create bundle directory at {:?}",
      bundle_directory
    )
  })?;

  let resources_dir = bundle_directory.join("Resources");
  let bin_dir = bundle_directory.join("MacOS");

  let bundle_icon_file: Option<PathBuf> =
    { create_icns_file(&resources_dir, settings).with_context(|| "Failed to create app icon")? };

  create_info_plist(&bundle_directory, bundle_icon_file, settings)
    .with_context(|| "Failed to create Info.plist")?;

  copy_frameworks_to_bundle(&bundle_directory, settings)
    .with_context(|| "Failed to bundle frameworks")?;

  settings.copy_resources(&resources_dir)?;

  settings
    .copy_binaries(&bin_dir)
    .with_context(|| "Failed to copy external binaries")?;

  copy_binaries_to_bundle(&bundle_directory, settings)?;

  let use_bootstrapper = settings.macos().use_bootstrapper.unwrap_or_default();
  if use_bootstrapper {
    create_bootstrapper(&bundle_directory, settings)
      .with_context(|| "Failed to create macOS bootstrapper")?;
  }
  Ok(vec![app_bundle_path])
}

// Copies the app's binaries to the bundle.
fn copy_binaries_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  let dest_dir = bundle_directory.join("MacOS");
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    common::copy_file(&bin_path, &dest_dir.join(bin.name()))
      .with_context(|| format!("Failed to copy binary from {:?}", bin_path))?;
  }
  Ok(())
}

// Creates the bootstrap script file.
fn create_bootstrapper(bundle_dir: &Path, settings: &Settings) -> crate::Result<()> {
  let file = &mut common::create_file(&bundle_dir.join("MacOS/__bootstrapper"))?;
  // Create a shell script to bootstrap the  $PATH for Tauri, so environments like node are available.
  write!(
    file,
    "#!/usr/bin/env sh
# This bootstraps the environment for Tauri, so environments are available.

if [ -e ~/.bash_profile ]
then 
  . ~/.bash_profile
fi
if [ -e ~/.zprofile ]
then 
  . ~/.zprofile
fi
if [ -e ~/.profile ]
then 
  . ~/.profile
fi
if [ -e ~/.bashrc ]
then 
  . ~/.bashrc
fi

if [ -e ~/.zshrc ]
then 
  . ~/.zshrc
fi

if pidof \"__bootstrapper\" >/dev/null; then
    exit 0
else
    exec \"`dirname \\\"$0\\\"`/{}\" $@ & disown
fi
exit 0",
    settings.product_name()
  )?;
  file.flush()?;

  // We have to make the __bootstrapper executable, or the bundle will not work
  Command::new("chmod")
    .arg("+x")
    .arg("__bootstrapper")
    .current_dir(&bundle_dir.join("MacOS/"))
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  Ok(())
}

// Creates the Info.plist file.
fn create_info_plist(
  bundle_dir: &Path,
  bundle_icon_file: Option<PathBuf>,
  settings: &Settings,
) -> crate::Result<()> {
  let build_number = chrono::Utc::now().format("%Y%m%d.%H%M%S");
  let file = &mut common::create_file(&bundle_dir.join("Info.plist"))?;
  let use_bootstrapper = settings.macos().use_bootstrapper.unwrap_or_default();
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
    settings.product_name()
  )?;
  write!(
    file,
    "  <key>CFBundleExecutable</key>\n  <string>{}</string>\n",
    if use_bootstrapper {
      "__bootstrapper"
    } else {
      settings.main_binary_name()
    }
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
    settings.product_name()
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
      category.macos_application_category_type()
    )?;
  }
  if let Some(version) = &settings.macos().minimum_system_version {
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

  if let Some(exception_domain) = &settings.macos().exception_domain {
    write!(
      file,
      "  <key>NSAppTransportSecurity</key>\n  \
      <dict>\n  \
          <key>NSExceptionDomains</key>\n  \
          <dict>\n  \
              <key>{}</key>\n  \
              <dict>\n  \
                  <key>NSExceptionAllowsInsecureHTTPLoads</key>\n  \
                  <true/>\n  \
                  <key>NSIncludesSubdomains</key>\n  \
                  <true/>\n  \
              </dict>\n  \
          </dict>\n  \
      </dict>",
      exception_domain
    )?;
  }

  write!(file, "</dict>\n</plist>\n")?;
  file.flush()?;
  Ok(())
}

// Copies the framework under `{src_dir}/{framework}.framework` to `{dest_dir}/{framework}.framework`.
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

// Copies the macOS application bundle frameworks to the .app
fn copy_frameworks_to_bundle(bundle_directory: &Path, settings: &Settings) -> crate::Result<()> {
  let frameworks = settings
    .macos()
    .frameworks
    .as_ref()
    .cloned()
    .unwrap_or_default();
  if frameworks.is_empty() {
    return Ok(());
  }
  let dest_dir = bundle_directory.join("Frameworks");
  fs::create_dir_all(&bundle_directory)
    .with_context(|| format!("Failed to create Frameworks directory at {:?}", dest_dir))?;
  for framework in frameworks.iter() {
    if framework.ends_with(".framework") {
      let src_path = PathBuf::from(framework);
      let src_name = src_path
        .file_name()
        .expect("Couldn't get framework filename");
      common::copy_dir(&src_path, &dest_dir.join(&src_name))?;
      continue;
    } else if framework.contains('/') {
      return Err(crate::Error::GenericError(format!(
        "Framework path should have .framework extension: {}",
        framework
      )));
    }
    if let Some(home_dir) = dirs_next::home_dir() {
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
    return Err(crate::Error::GenericError(format!(
      "Could not locate framework: {}",
      framework
    )));
  }
  Ok(())
}

// Given a list of icon files, try to produce an ICNS file in the resources
// directory and return the path to it.  Returns `Ok(None)` if no usable icons
// were provided.
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
      dest_path.push(icon_path.file_name().expect("Could not get icon filename"));
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
    let icon = icon.resize_exact(
      next_size_down,
      next_size_down,
      image::imageops::FilterType::Lanczos3,
    );
    add_icon_to_family(icon, density, &mut family)?;
  }

  if !family.is_empty() {
    fs::create_dir_all(resources_dir)?;
    let mut dest_path = resources_dir.clone();
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
  icns::Image::from_data(pixel_format, img.width(), img.height(), img.to_bytes())
}
