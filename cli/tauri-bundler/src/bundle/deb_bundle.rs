// The structure of a Debian package looks something like this:
//
// foobar_1.2.3_i386.deb   # Actually an ar archive
//     debian-binary           # Specifies deb format version (2.0 in our case)
//     control.tar.gz          # Contains files controlling the installation:
//         control                  # Basic package metadata
//         md5sums                  # Checksums for files in data.tar.gz below
//         postinst                 # Post-installation script (optional)
//         prerm                    # Pre-uninstallation script (optional)
//     data.tar.gz             # Contains files to be installed:
//         usr/bin/foobar                            # Binary executable file
//         usr/share/applications/foobar.desktop     # Desktop file (for apps)
//         usr/share/icons/hicolor/...               # Icon files (for apps)
//         usr/lib/foobar/...                        # Other resource files
//
// For cargo-bundle, we put bundle resource files under /usr/lib/package_name/,
// and then generate the desktop file and control file from the bundle
// metadata, as well as generating the md5sums file.  Currently we do not
// generate postinst or prerm files.

use super::common;
use crate::Settings;

use anyhow::Context;
use image::{self, png::PngDecoder, GenericImageView, ImageDecoder};
use libflate::gzip;
use std::process::{Command, Stdio};
use walkdir::WalkDir;

use std::{
  collections::BTreeSet,
  ffi::OsStr,
  fs::{self, File},
  io::{self, Write},
  path::{Path, PathBuf},
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the DEB was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    "x86" => "i386",
    "x86_64" => "amd64",
    other => other,
  };
  let package_base_name = format!(
    "{}_{}_{}",
    settings.main_binary_name(),
    settings.version_string(),
    arch
  );
  let package_name = format!("{}.deb", package_base_name);
  common::print_bundling(&package_name)?;
  let base_dir = settings.project_out_directory().join("bundle/deb");
  let package_dir = base_dir.join(&package_base_name);
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir)
      .with_context(|| format!("Failed to remove old {}", package_base_name))?;
  }
  let package_path = base_dir.join(package_name);

  let data_dir = generate_data(settings, &package_dir)
    .with_context(|| "Failed to build data folders and files")?;
  // Generate control files.
  let control_dir = package_dir.join("control");
  generate_control_file(settings, arch, &control_dir, &data_dir)
    .with_context(|| "Failed to create control file")?;
  generate_md5sums(&control_dir, &data_dir).with_context(|| "Failed to create md5sums file")?;

  // Generate `debian-binary` file; see
  // http://www.tldp.org/HOWTO/Debian-Binary-Package-Building-HOWTO/x60.html#AEN66
  let debian_binary_path = package_dir.join("debian-binary");
  create_file_with_data(&debian_binary_path, "2.0\n")
    .with_context(|| "Failed to create debian-binary file")?;

  // Apply tar/gzip/ar to create the final package file.
  let control_tar_gz_path =
    tar_and_gzip_dir(control_dir).with_context(|| "Failed to tar/gzip control directory")?;
  let data_tar_gz_path =
    tar_and_gzip_dir(data_dir).with_context(|| "Failed to tar/gzip data directory")?;
  create_archive(
    vec![debian_binary_path, control_tar_gz_path, data_tar_gz_path],
    &package_path,
  )
  .with_context(|| "Failed to create package archive")?;
  Ok(vec![package_path])
}

/// Generate the debian data folders and files.
pub fn generate_data(settings: &Settings, package_dir: &Path) -> crate::Result<PathBuf> {
  // Generate data files.
  let data_dir = package_dir.join("data");
  let bin_dir = data_dir.join("usr/bin");

  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    common::copy_file(&bin_path, &bin_dir.join(bin.name()))
      .with_context(|| format!("Failed to copy binary from {:?}", bin_path))?;
  }

  transfer_resource_files(settings, &data_dir).with_context(|| "Failed to copy resource files")?;

  settings
    .copy_binaries(&bin_dir)
    .with_context(|| "Failed to copy external binaries")?;

  generate_icon_files(settings, &data_dir).with_context(|| "Failed to create icon files")?;
  generate_desktop_file(settings, &data_dir).with_context(|| "Failed to create desktop file")?;

  let use_bootstrapper = settings.deb().use_bootstrapper.unwrap_or_default();
  if use_bootstrapper {
    generate_bootstrap_file(settings, &data_dir)
      .with_context(|| "Failed to generate bootstrap file")?;
  }

  Ok(data_dir)
}

/// Generates the bootstrap script file.
fn generate_bootstrap_file(settings: &Settings, data_dir: &Path) -> crate::Result<()> {
  let bin_name = settings.main_binary_name();
  let bin_dir = data_dir.join("usr/bin");

  let bootstrap_file_name = format!("__{}-bootstrapper", bin_name);
  let bootstrapper_file_path = bin_dir.join(bootstrap_file_name.clone());
  let bootstrapper_file = &mut common::create_file(&bootstrapper_file_path)?;
  write!(
    bootstrapper_file,
    "#!/usr/bin/env sh
# This bootstraps the environment for Tauri, so environments are available.
export NVM_DIR=\"$([ -z \"${{XDG_CONFIG_HOME-}}\" ] && printf %s \"${{HOME}}/.nvm\" || printf %s \"${{XDG_CONFIG_HOME}}/nvm\")\"
[ -s \"$NVM_DIR/nvm.sh\" ] && . \"$NVM_DIR/nvm.sh\"

if [ -e ~/.bash_profile ]
then
    source ~/.bash_profile
fi
if [ -e ~/.zprofile ]
then
    source ~/.zprofile
fi
if [ -e ~/.profile ]
then
    source ~/.profile
fi
if [ -e ~/.bashrc ]
then
    source ~/.bashrc
fi
if [ -e ~/.zshrc ]
then
    source ~/.zshrc
fi

echo $PATH

source /etc/profile

if pidof -x \"{}\" >/dev/null; then
    exit 0
else
    Exec=/usr/bin/env /usr/bin/{} $@ & disown
fi
exit 0",
    bootstrap_file_name, bin_name
  )?;
  bootstrapper_file.flush()?;

  Command::new("chmod")
    .arg("+x")
    .arg(bootstrap_file_name)
    .current_dir(&bin_dir)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

  Ok(())
}

/// Generate the application desktop file and store it under the `data_dir`.
fn generate_desktop_file(settings: &Settings, data_dir: &Path) -> crate::Result<()> {
  let bin_name = settings.main_binary_name();
  let desktop_file_name = format!("{}.desktop", bin_name);
  let desktop_file_path = data_dir
    .join("usr/share/applications")
    .join(desktop_file_name);
  let file = &mut common::create_file(&desktop_file_path)?;
  // For more information about the format of this file, see
  // https://developer.gnome.org/integration-guide/stable/desktop-files.html.en
  writeln!(file, "[Desktop Entry]")?;
  if let Some(category) = settings.app_category() {
    writeln!(file, "Categories={}", category.gnome_desktop_categories())?;
  } else {
    writeln!(file, "Categories=")?;
  }
  if !settings.short_description().is_empty() {
    writeln!(file, "Comment={}", settings.short_description())?;
  }
  let use_bootstrapper = settings.deb().use_bootstrapper.unwrap_or_default();
  writeln!(
    file,
    "Exec={}",
    if use_bootstrapper {
      format!("__{}-bootstrapper", bin_name)
    } else {
      bin_name.to_string()
    }
  )?;
  writeln!(file, "Icon={}", bin_name)?;
  writeln!(file, "Name={}", settings.product_name())?;
  writeln!(file, "Terminal=false")?;
  writeln!(file, "Type=Application")?;
  Ok(())
}

/// Generates the debian control file and stores it under the `control_dir`.
fn generate_control_file(
  settings: &Settings,
  arch: &str,
  control_dir: &Path,
  data_dir: &Path,
) -> crate::Result<()> {
  // For more information about the format of this file, see
  // https://www.debian.org/doc/debian-policy/ch-controlfields.html
  let dest_path = control_dir.join("control");
  let mut file = common::create_file(&dest_path)?;
  writeln!(
    &mut file,
    "Package: {}",
    str::replace(settings.product_name(), " ", "-").to_ascii_lowercase()
  )?;
  writeln!(&mut file, "Version: {}", settings.version_string())?;
  writeln!(&mut file, "Architecture: {}", arch)?;
  writeln!(&mut file, "Installed-Size: {}", total_dir_size(data_dir)?)?;
  let authors = settings
    .authors_comma_separated()
    .unwrap_or_else(String::new);
  writeln!(&mut file, "Maintainer: {}", authors)?;
  if !settings.homepage_url().is_empty() {
    writeln!(&mut file, "Homepage: {}", settings.homepage_url())?;
  }
  let dependencies = settings.deb().depends.as_ref().cloned().unwrap_or_default();
  if !dependencies.is_empty() {
    writeln!(&mut file, "Depends: {}", dependencies.join(", "))?;
  }
  let mut short_description = settings.short_description().trim();
  if short_description.is_empty() {
    short_description = "(none)";
  }
  let mut long_description = settings.long_description().unwrap_or("").trim();
  if long_description.is_empty() {
    long_description = "(none)";
  }
  writeln!(&mut file, "Description: {}", short_description)?;
  for line in long_description.lines() {
    let line = line.trim();
    if line.is_empty() {
      writeln!(&mut file, " .")?;
    } else {
      writeln!(&mut file, " {}", line)?;
    }
  }
  file.flush()?;
  Ok(())
}

/// Create an `md5sums` file in the `control_dir` containing the MD5 checksums
/// for each file within the `data_dir`.
fn generate_md5sums(control_dir: &Path, data_dir: &Path) -> crate::Result<()> {
  let md5sums_path = control_dir.join("md5sums");
  let mut md5sums_file = common::create_file(&md5sums_path)?;
  for entry in WalkDir::new(data_dir) {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      continue;
    }
    let mut file = File::open(path)?;
    let mut hash = md5::Context::new();
    io::copy(&mut file, &mut hash)?;
    for byte in hash.compute().iter() {
      write!(md5sums_file, "{:02x}", byte)?;
    }
    let rel_path = path.strip_prefix(data_dir)?;
    let path_str = rel_path.to_str().ok_or_else(|| {
      let msg = format!("Non-UTF-8 path: {:?}", rel_path);
      io::Error::new(io::ErrorKind::InvalidData, msg)
    })?;
    writeln!(md5sums_file, "  {}", path_str)?;
  }
  Ok(())
}

/// Copy the bundle's resource files into an appropriate directory under the
/// `data_dir`.
fn transfer_resource_files(settings: &Settings, data_dir: &Path) -> crate::Result<()> {
  let resource_dir = data_dir.join("usr/lib").join(settings.main_binary_name());
  settings.copy_resources(&resource_dir)
}

/// Generate the icon files and store them under the `data_dir`.
fn generate_icon_files(settings: &Settings, data_dir: &PathBuf) -> crate::Result<()> {
  let base_dir = data_dir.join("usr/share/icons/hicolor");
  let get_dest_path = |width: u32, height: u32, is_high_density: bool| {
    base_dir.join(format!(
      "{}x{}{}/apps/{}.png",
      width,
      height,
      if is_high_density { "@2x" } else { "" },
      settings.main_binary_name()
    ))
  };
  let mut sizes = BTreeSet::new();
  // Prefer PNG files.
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;
    if icon_path.extension() != Some(OsStr::new("png")) {
      continue;
    }
    let decoder = PngDecoder::new(File::open(&icon_path)?)?;
    let width = decoder.dimensions().0;
    let height = decoder.dimensions().1;
    let is_high_density = common::is_retina(&icon_path);
    if !sizes.contains(&(width, height, is_high_density)) {
      sizes.insert((width, height, is_high_density));
      let dest_path = get_dest_path(width, height, is_high_density);
      common::copy_file(&icon_path, &dest_path)?;
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
        let is_high_density = icon_type.pixel_density() > 1;
        if !sizes.contains(&(width, height, is_high_density)) {
          sizes.insert((width, height, is_high_density));
          let dest_path = get_dest_path(width, height, is_high_density);
          let icon = icon_family.get_icon_with_type(icon_type)?;
          icon.write_png(common::create_file(&dest_path)?)?;
        }
      }
    } else {
      let icon = image::open(&icon_path)?;
      let (width, height) = icon.dimensions();
      let is_high_density = common::is_retina(&icon_path);
      if !sizes.contains(&(width, height, is_high_density)) {
        sizes.insert((width, height, is_high_density));
        let dest_path = get_dest_path(width, height, is_high_density);
        icon.write_to(
          &mut common::create_file(&dest_path)?,
          image::ImageOutputFormat::Png,
        )?;
      }
    }
  }
  Ok(())
}

/// Create an empty file at the given path, creating any parent directories as
/// needed, then write `data` into the file.
fn create_file_with_data<P: AsRef<Path>>(path: P, data: &str) -> crate::Result<()> {
  let mut file = common::create_file(path.as_ref())?;
  file.write_all(data.as_bytes())?;
  file.flush()?;
  Ok(())
}

/// Computes the total size, in bytes, of the given directory and all of its
/// contents.
fn total_dir_size(dir: &Path) -> crate::Result<u64> {
  let mut total: u64 = 0;
  for entry in WalkDir::new(&dir) {
    total += entry?.metadata()?.len();
  }
  Ok(total)
}

/// Writes a tar file to the given writer containing the given directory.
fn create_tar_from_dir<P: AsRef<Path>, W: Write>(src_dir: P, dest_file: W) -> crate::Result<W> {
  let src_dir = src_dir.as_ref();
  let mut tar_builder = tar::Builder::new(dest_file);
  for entry in WalkDir::new(&src_dir) {
    let entry = entry?;
    let src_path = entry.path();
    if src_path == src_dir {
      continue;
    }
    let dest_path = src_path.strip_prefix(&src_dir)?;
    if entry.file_type().is_dir() {
      tar_builder.append_dir(dest_path, src_path)?;
    } else {
      let mut src_file = fs::File::open(src_path)?;
      tar_builder.append_file(dest_path, &mut src_file)?;
    }
  }
  let dest_file = tar_builder.into_inner()?;
  Ok(dest_file)
}

/// Creates a `.tar.gz` file from the given directory (placing the new file
/// within the given directory's parent directory), then deletes the original
/// directory and returns the path to the new file.
fn tar_and_gzip_dir<P: AsRef<Path>>(src_dir: P) -> crate::Result<PathBuf> {
  let src_dir = src_dir.as_ref();
  let dest_path = src_dir.with_extension("tar.gz");
  let dest_file = common::create_file(&dest_path)?;
  let gzip_encoder = gzip::Encoder::new(dest_file)?;
  let gzip_encoder = create_tar_from_dir(src_dir, gzip_encoder)?;
  let mut dest_file = gzip_encoder.finish().into_result()?;
  dest_file.flush()?;
  Ok(dest_path)
}

/// Creates an `ar` archive from the given source files and writes it to the
/// given destination path.
fn create_archive(srcs: Vec<PathBuf>, dest: &Path) -> crate::Result<()> {
  let mut builder = ar::Builder::new(common::create_file(&dest)?);
  for path in &srcs {
    builder.append_path(path)?;
  }
  builder.into_inner()?.flush()?;
  Ok(())
}
