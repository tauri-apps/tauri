// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

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

use super::freedesktop;
use crate::{
  Settings,
  bundle::settings::Arch,
  error::{Context, ErrorExt},
  utils::{CommandExt, fs_utils},
};
use flate2::{Compression, write::GzEncoder};
use tar::HeaderMode;
use walkdir::WalkDir;

use std::{
  fs::{self, File, OpenOptions},
  io::{self, Write},
  os::unix::fs::{MetadataExt, OpenOptionsExt, symlink},
  path::{Path, PathBuf},
  process::Command,
};

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the DEB was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = match settings.binary_arch() {
    Arch::X86_64 => "amd64",
    Arch::X86 => "i386",
    Arch::AArch64 => "arm64",
    Arch::Armhf => "armhf",
    Arch::Armel => "armel",
    Arch::Riscv64 => "riscv64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };
  let package_base_name = format!(
    "{}_{}_{}",
    settings.product_name(),
    settings.version_string(),
    arch
  );
  let package_name = format!("{package_base_name}.deb");

  let base_dir = settings.project_out_directory().join("bundle/deb");
  let package_dir = base_dir.join(&package_base_name);
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir).fs_context(
      "Failed to Remove old package directory",
      package_dir.clone(),
    )?;
  }
  let package_path = base_dir.join(&package_name);

  log::info!(action = "Bundling"; "{} ({})", package_name, package_path.display());

  let (data_dir, _) =
    generate_data(settings, &package_dir).context("Failed to build data folders and files")?;
  fs_utils::copy_custom_files(&settings.deb().files, &data_dir)
    .context("Failed to copy custom files")?;

  // Handle CEF support if cef_path is set,
  // using https://github.com/chromiumembedded/cef/blob/master/tools/distrib/linux/README.redistrib.txt as a reference
  //
  // Dealing with rpath or LD_LIBRARY_PATH is annoying so we'll somewhat follow the approach of spotify(cef) and electron apps and move the binary out of /usr/bin for now.
  // This still requires adding $ORIGIN to RUNPATH, which we currently do in tauri-build.
  if let Some(cef_path) = settings.bundle_settings().cef_path.as_ref() {
    let share_dir = data_dir.join("usr/share").join(settings.product_name());
    fs::create_dir_all(&share_dir)?;

    // TODO: we may have to copy all binaries.
    let main_bin = settings
      .binaries()
      .iter()
      .find(|b| b.main())
      .expect("one main binary should always exist")
      .name();

    fs::rename(
      data_dir.join("usr/bin").join(main_bin),
      share_dir.join(main_bin),
    )?;

    symlink(
      format!("../share/{}/{main_bin}", settings.product_name()),
      data_dir.join("usr/bin").join(main_bin),
    )?;

    let cef_files = [
      // required
      "libcef.so",
      "icudtl.dat",
      "v8_context_snapshot.bin",
      // required end
      // "optional" - but not really since we want support for all of this
      "chrome_100_percent.pak",
      "chrome_200_percent.pak",
      "resources.pak",
      // ANGEL support
      "libEGL.so",
      "libGLESv2.so",
      // SwANGLE support
      "libvk_swiftshader.so",
      "vk_swiftshader_icd.json",
      "libvulkan.so.1",
      // sandbox
      "chrome-sandbox",
    ];

    for f in cef_files {
      let file_dest = share_dir.join(f);
      fs::copy(cef_path.join(f), &file_dest)?;
      if f.ends_with(".so") {
        // since libcef.so is 1.5GB unstripped we will error out if strip fails.
        Command::new("strip").arg(file_dest).output_ok()?;
      }
    }
    // TODO: Check if/when we need the other lang files
    let locales = [
      "en-US.pak",
      "en-US_FEMININE.pak",
      "en-US_MASCULINE.pak",
      "en-US_NEUTER.pak",
    ];

    let cef_path = cef_path.join("locales");
    let share_dir = share_dir.join("locales");
    fs::create_dir_all(&share_dir)?;

    for f in locales {
      fs::copy(cef_path.join(f), share_dir.join(f))?;
    }
    // cef_path and share_dir still point to locales!
  }

  // Generate control files.
  let control_dir = package_dir.join("control");
  generate_control_file(settings, arch, &control_dir, &data_dir)
    .context("Failed to create control file")?;
  generate_scripts(settings, &control_dir).context("Failed to create control scripts")?;
  generate_md5sums(&control_dir, &data_dir).context("Failed to create md5sums file")?;

  // Generate `debian-binary` file; see
  // http://www.tldp.org/HOWTO/Debian-Binary-Package-Building-HOWTO/x60.html#AEN66
  let debian_binary_path = package_dir.join("debian-binary");
  create_file_with_data(&debian_binary_path, "2.0\n")
    .context("Failed to create debian-binary file")?;

  log::info!(action = "Bundling"; "Creating .deb archive...");

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
pub fn generate_data(
  settings: &Settings,
  package_dir: &Path,
) -> crate::Result<(PathBuf, Vec<freedesktop::Icon>)> {
  // Generate data files.
  let data_dir = package_dir.join("data");
  let bin_dir = data_dir.join("usr/bin");

  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    let trgt = bin_dir.join(bin.name());
    fs_utils::copy_file(&bin_path, &trgt)
      .with_context(|| format!("Failed to copy binary from {bin_path:?} to {trgt:?}"))?;
  }

  copy_resource_files(settings, &data_dir).with_context(|| "Failed to copy resource files")?;

  settings
    .copy_binaries(&bin_dir)
    .with_context(|| "Failed to copy external binaries")?;

  let icons = freedesktop::copy_icon_files(settings, &data_dir)
    .with_context(|| "Failed to create icon files")?;
  freedesktop::generate_desktop_file(settings, &settings.deb().desktop_template, &data_dir)
    .with_context(|| "Failed to create desktop file")?;
  generate_changelog_file(settings, &data_dir)
    .with_context(|| "Failed to create changelog.gz file")?;

  Ok((data_dir, icons))
}

/// Generate the Changelog file by compressing, to be stored at /usr/share/doc/package-name/changelog.gz. See
/// <https://www.debian.org/doc/debian-policy/ch-docs.html#changelog-files-and-release-notes>
fn generate_changelog_file(settings: &Settings, data_dir: &Path) -> crate::Result<()> {
  if let Some(changelog_src_path) = &settings.deb().changelog {
    let mut src_file = File::open(changelog_src_path)?;
    let product_name = settings.product_name();
    let dest_path = data_dir.join(format!("usr/share/doc/{product_name}/changelog.gz"));

    let changelog_file = fs_utils::create_file(&dest_path)?;
    let mut gzip_encoder = GzEncoder::new(changelog_file, Compression::new(9));
    io::copy(&mut src_file, &mut gzip_encoder)?;

    let mut changelog_file = gzip_encoder.finish()?;
    changelog_file.flush()?;
  }
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
  let mut file = fs_utils::create_file(&dest_path)?;
  let package = heck::AsKebabCase(settings.product_name());
  writeln!(file, "Package: {package}")?;
  writeln!(file, "Version: {}", settings.version_string())?;
  writeln!(file, "Architecture: {arch}")?;
  // Installed-Size must be divided by 1024, see https://www.debian.org/doc/debian-policy/ch-controlfields.html#installed-size
  writeln!(file, "Installed-Size: {}", total_dir_size(data_dir)? / 1024)?;
  let authors = settings
    .authors_comma_separated()
    .or_else(|| settings.publisher().map(ToString::to_string))
    .unwrap_or_else(|| {
      settings
        .bundle_identifier()
        .split('.')
        .nth(1)
        .unwrap_or(settings.bundle_identifier())
        .to_string()
    });

  writeln!(file, "Maintainer: {authors}")?;
  if let Some(section) = &settings.deb().section {
    writeln!(file, "Section: {section}")?;
  }
  if let Some(priority) = &settings.deb().priority {
    writeln!(file, "Priority: {priority}")?;
  } else {
    writeln!(file, "Priority: optional")?;
  }

  if let Some(homepage) = settings.homepage_url() {
    writeln!(file, "Homepage: {homepage}")?;
  }

  let dependencies = settings.deb().depends.as_ref().cloned().unwrap_or_default();
  if !dependencies.is_empty() {
    writeln!(file, "Depends: {}", dependencies.join(", "))?;
  }
  let dependencies = settings
    .deb()
    .recommends
    .as_ref()
    .cloned()
    .unwrap_or_default();
  if !dependencies.is_empty() {
    writeln!(file, "Recommends: {}", dependencies.join(", "))?;
  }
  let provides = settings
    .deb()
    .provides
    .as_ref()
    .cloned()
    .unwrap_or_default();
  if !provides.is_empty() {
    writeln!(file, "Provides: {}", provides.join(", "))?;
  }
  let conflicts = settings
    .deb()
    .conflicts
    .as_ref()
    .cloned()
    .unwrap_or_default();
  if !conflicts.is_empty() {
    writeln!(file, "Conflicts: {}", conflicts.join(", "))?;
  }
  let replaces = settings
    .deb()
    .replaces
    .as_ref()
    .cloned()
    .unwrap_or_default();
  if !replaces.is_empty() {
    writeln!(file, "Replaces: {}", replaces.join(", "))?;
  }
  let mut short_description = settings.short_description().trim();
  if short_description.is_empty() {
    short_description = "(none)";
  }
  let mut long_description = settings.long_description().unwrap_or("").trim();
  if long_description.is_empty() {
    long_description = "(none)";
  }
  writeln!(file, "Description: {short_description}")?;
  for line in long_description.lines() {
    let line = line.trim();
    if line.is_empty() {
      writeln!(file, " .")?;
    } else {
      writeln!(file, " {line}")?;
    }
  }
  file.flush()?;
  Ok(())
}

fn generate_scripts(settings: &Settings, control_dir: &Path) -> crate::Result<()> {
  if let Some(script_path) = &settings.deb().pre_install_script {
    let dest_path = control_dir.join("preinst");
    create_script_file_from_path(script_path, &dest_path)?
  }

  if let Some(script_path) = &settings.deb().post_install_script {
    let dest_path = control_dir.join("postinst");
    create_script_file_from_path(script_path, &dest_path)?
  }

  if let Some(script_path) = &settings.deb().pre_remove_script {
    let dest_path = control_dir.join("prerm");
    create_script_file_from_path(script_path, &dest_path)?
  }

  if let Some(script_path) = &settings.deb().post_remove_script {
    let dest_path = control_dir.join("postrm");
    create_script_file_from_path(script_path, &dest_path)?
  }
  Ok(())
}

fn create_script_file_from_path(from: &PathBuf, to: &PathBuf) -> crate::Result<()> {
  let mut from = File::open(from)?;
  let mut file = OpenOptions::new()
    .create(true)
    .truncate(true)
    .write(true)
    .mode(0o755)
    .open(to)?;
  std::io::copy(&mut from, &mut file)?;
  Ok(())
}

/// Create an `md5sums` file in the `control_dir` containing the MD5 checksums
/// for each file within the `data_dir`.
fn generate_md5sums(control_dir: &Path, data_dir: &Path) -> crate::Result<()> {
  let md5sums_path = control_dir.join("md5sums");
  let mut md5sums_file = fs_utils::create_file(&md5sums_path)?;
  for entry in WalkDir::new(data_dir) {
    let entry = entry?;
    let path = entry.path();
    if path.is_dir() {
      continue;
    }
    let mut file = File::open(path)?;
    let mut hash = md5::Context::new();
    io::copy(&mut file, &mut hash)?;
    for byte in hash.finalize().iter() {
      write!(md5sums_file, "{byte:02x}")?;
    }
    let rel_path = path.strip_prefix(data_dir)?;
    let path_str = rel_path.to_str().ok_or_else(|| {
      let msg = format!("Non-UTF-8 path: {rel_path:?}");
      io::Error::new(io::ErrorKind::InvalidData, msg)
    })?;
    writeln!(md5sums_file, "  {path_str}")?;
  }
  Ok(())
}

/// Copy the bundle's resource files into an appropriate directory under the
/// `data_dir`.
fn copy_resource_files(settings: &Settings, data_dir: &Path) -> crate::Result<()> {
  let resource_dir = data_dir.join("usr/lib").join(settings.product_name());
  settings.copy_resources(&resource_dir)
}

/// Create an empty file at the given path, creating any parent directories as
/// needed, then write `data` into the file.
fn create_file_with_data<P: AsRef<Path>>(path: P, data: &str) -> crate::Result<()> {
  let mut file = fs_utils::create_file(path.as_ref())?;
  file.write_all(data.as_bytes())?;
  file.flush()?;
  Ok(())
}

/// Computes the total size, in bytes, of the given directory and all of its
/// contents.
fn total_dir_size(dir: &Path) -> crate::Result<u64> {
  let mut total: u64 = 0;
  for entry in WalkDir::new(dir) {
    total += entry?.metadata()?.len();
  }
  Ok(total)
}

/// Writes a tar file to the given writer containing the given directory.
fn create_tar_from_dir<P: AsRef<Path>, W: Write>(src_dir: P, dest_file: W) -> crate::Result<W> {
  let src_dir = src_dir.as_ref();
  let mut tar_builder = tar::Builder::new(dest_file);
  for entry in WalkDir::new(src_dir) {
    let entry = entry?;
    let src_path = entry.path();
    if src_path == src_dir {
      continue;
    }

    let dest_path = src_path.strip_prefix(src_dir)?;

    let stat_metadata = fs::symlink_metadata(src_path)?;
    // TODO: This should probably only trigger for the main binary for cef apps
    if stat_metadata.is_symlink() {
      let mut header = tar::Header::new_gnu();
      header.set_metadata_in_mode(&stat_metadata, HeaderMode::Deterministic);
      header.set_mtime(stat_metadata.mtime() as u64);
      header.set_entry_type(tar::EntryType::Symlink);
      let target_path = fs::read_link(src_path)?;
      tar_builder.append_link(&mut header, dest_path, target_path)?;
    } else {
      let stat = fs::metadata(src_path)?;
      let mut header = tar::Header::new_gnu();
      header.set_metadata_in_mode(&stat, HeaderMode::Deterministic);
      header.set_mtime(stat.mtime() as u64);
      if src_path.ends_with("chrome-sandbox") {
        header.set_mode(0o4755);
      }

      if entry.file_type().is_dir() {
        tar_builder.append_data(&mut header, dest_path, &mut io::empty())?;
      } else {
        let mut src_file = fs::File::open(src_path)?;
        tar_builder.append_data(&mut header, dest_path, &mut src_file)?;
      }
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
  let dest_file = fs_utils::create_file(&dest_path)?;
  let gzip_encoder = GzEncoder::new(dest_file, Compression::default());
  let gzip_encoder = create_tar_from_dir(src_dir, gzip_encoder)?;
  let mut dest_file = gzip_encoder.finish()?;
  dest_file.flush()?;
  Ok(dest_path)
}

/// Creates an `ar` archive from the given source files and writes it to the
/// given destination path.
fn create_archive(srcs: Vec<PathBuf>, dest: &Path) -> crate::Result<()> {
  let mut builder = ar::Builder::new(fs_utils::create_file(dest)?);
  for path in &srcs {
    builder.append_path(path)?;
  }
  builder.into_inner()?.flush()?;
  Ok(())
}
