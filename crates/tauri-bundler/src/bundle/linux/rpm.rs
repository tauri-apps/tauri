// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Settings, bundle::settings::Arch, error::ErrorExt, utils::CommandExt};

use rpm::{self, Dependency, FileOptions, signature::pgp};
use std::{
  env, fs,
  path::{Path, PathBuf},
  process::Command,
};
use tauri_utils::config::RpmCompression;

use super::freedesktop;

// https://docs.fedoraproject.org/en-US/packaging-guidelines/Versioning/
// TODO: this may not cover it perfectly yet, it's just a hotfix for prerelease semver
fn to_rpm_version(version: &str) -> String {
  match semver::Version::parse(version) {
    Ok(v) if !v.pre.is_empty() => {
      let pre = v.pre.as_str().replace('-', ".");
      let mut rpm = format!("{}.{}.{}~{}", v.major, v.minor, v.patch, pre);
      if !v.build.is_empty() {
        rpm.push('+');
        rpm.push_str(v.build.as_str());
      }
      rpm
    }
    _ => version.to_string(),
  }
}

/// Bundles the project.
/// Returns a vector of PathBuf that shows where the RPM was created.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let product_name = settings.product_name();
  let version = settings.version_string();
  let release = match settings.rpm().release.as_str() {
    "" => "1", // Considered the default. If left empty, you get file with "-.".
    v => v,
  };
  let epoch = settings.rpm().epoch;
  let arch = match settings.binary_arch() {
    Arch::X86_64 => "x86_64",
    Arch::X86 => "i386",
    Arch::AArch64 => "aarch64",
    Arch::Armhf => "armhfp",
    Arch::Armel => "armel",
    Arch::Riscv64 => "riscv64",
    target => {
      return Err(crate::Error::ArchError(format!(
        "Unsupported architecture: {target:?}"
      )));
    }
  };

  let summary = settings.short_description().trim();

  let package_base_name = format!("{product_name}-{version}-{release}.{arch}");
  let package_name = format!("{package_base_name}.rpm");

  let base_dir = settings.project_out_directory().join("bundle/rpm");
  let package_dir = base_dir.join(&package_base_name);
  if package_dir.exists() {
    fs::remove_dir_all(&package_dir).fs_context(
      "Failed to remove old package directory",
      package_dir.clone(),
    )?;
  }
  fs::create_dir_all(&package_dir)
    .fs_context("Failed to create package directory", package_dir.clone())?;
  let package_path = base_dir.join(&package_name);

  log::info!(action = "Bundling"; "{} ({})", package_name, package_path.display());

  let license = settings.license().unwrap_or_default();
  let name = heck::AsKebabCase(settings.product_name()).to_string();

  let compression = settings
    .rpm()
    .compression
    .map(|c| match c {
      RpmCompression::Gzip { level } => rpm::CompressionWithLevel::Gzip(level),
      RpmCompression::Zstd { level } => rpm::CompressionWithLevel::Zstd(level),
      RpmCompression::Xz { level } => rpm::CompressionWithLevel::Xz(level),
      RpmCompression::Bzip2 { level } => rpm::CompressionWithLevel::Bzip2(level),
      _ => rpm::CompressionWithLevel::None,
    })
    .unwrap_or_default();

  let build_config = rpm::BuildConfig::default().compression(compression);

  let mut builder =
    rpm::PackageBuilder::new(&name, &to_rpm_version(version), &license, arch, summary);
  builder
    .using_config(build_config)
    .epoch(epoch)
    .release(release);

  if let Some(description) = settings.long_description() {
    builder.description(description);
  }

  if let Some(homepage) = settings.homepage_url() {
    builder.url(homepage);
  }

  // Add requirements
  for dep in settings.rpm().depends.as_ref().cloned().unwrap_or_default() {
    builder.requires(Dependency::any(dep));
  }

  // Add provides
  for dep in settings
    .rpm()
    .provides
    .as_ref()
    .cloned()
    .unwrap_or_default()
  {
    builder.provides(Dependency::any(dep));
  }

  // Add recommends
  for dep in settings
    .rpm()
    .recommends
    .as_ref()
    .cloned()
    .unwrap_or_default()
  {
    builder.recommends(Dependency::any(dep));
  }

  // Add conflicts
  for dep in settings
    .rpm()
    .conflicts
    .as_ref()
    .cloned()
    .unwrap_or_default()
  {
    builder.conflicts(Dependency::any(dep));
  }

  // Add obsoletes
  for dep in settings
    .rpm()
    .obsoletes
    .as_ref()
    .cloned()
    .unwrap_or_default()
  {
    builder.obsoletes(Dependency::any(dep));
  }

  // Add binaries
  for bin in settings.binaries() {
    let src = settings.binary_path(bin);
    let dest = Path::new("/usr/bin").join(bin.name());
    // This may cause issues when you try to submit the app to the distro repos but this is how apps like spotify do it as well (in .deb)
    if settings.bundle_settings().cef_path.is_some() && bin.main() {
      let cef_bin_dest = Path::new("/usr/share")
        .join(settings.product_name())
        .join(bin.name());
      builder.with_file(src, FileOptions::new(cef_bin_dest.to_string_lossy()))?;
      builder.with_symlink(
        FileOptions::symlink(
          dest.to_string_lossy(),
          cef_bin_dest.to_string_lossy().replace("/usr", ".."),
        )
        .mode(0o120555),
      )?;
    } else {
      builder.with_file(src, FileOptions::new(dest.to_string_lossy()))?;
    }
  }

  // Add external binaries
  for src in settings.external_binaries() {
    let src = src?;
    let dest = Path::new("/usr/bin").join(
      src
        .file_name()
        .expect("failed to extract external binary filename")
        .to_string_lossy()
        .replace(&format!("-{}", settings.target()), ""),
    );
    builder.with_file(&src, FileOptions::new(dest.to_string_lossy()))?;
  }

  // Add scripts
  if let Some(script_path) = &settings.rpm().pre_install_script {
    let script = fs::read_to_string(script_path)?;
    builder.pre_install_script(script);
  }

  if let Some(script_path) = &settings.rpm().post_install_script {
    let script = fs::read_to_string(script_path)?;
    builder.post_install_script(script);
  }

  if let Some(script_path) = &settings.rpm().pre_remove_script {
    let script = fs::read_to_string(script_path)?;
    builder.pre_uninstall_script(script);
  }

  if let Some(script_path) = &settings.rpm().post_remove_script {
    let script = fs::read_to_string(script_path)?;
    builder.post_uninstall_script(script);
  }

  // Add resources and/or prepare for CEF files
  if settings.resource_files().count() > 0 || settings.bundle_settings().cef_path.is_some() {
    let resource_dir = Path::new("/usr/lib").join(settings.product_name());
    builder.with_dir_entry(FileOptions::dir(resource_dir.to_string_lossy()).permissions(0o755))?;
    // Then add the resources files in that directory
    for resource in settings.resource_files().iter() {
      let resource = resource?;
      let dest = resource_dir.join(resource.target());
      builder.with_file(resource.path(), FileOptions::new(dest.to_string_lossy()))?;
    }
  }
  // Handle CEF support if cef_path is set,
  // using https://github.com/chromiumembedded/cef/blob/master/tools/distrib/linux/README.redistrib.txt as a reference
  //
  // Dealing with rpath or LD_LIBRARY_PATH is annoying so we'll somewhat follow the spotify approach and move the binary out of /usr/bin for now.
  // This still requires adding $ORIGIN to RUNPATH, which we currently do in tauri-build.
  // TODO: This may cause issues when you try to submit the app to the distro repos but we can revisit this later.
  if let Some(cef_path) = settings.bundle_settings().cef_path.as_ref() {
    let cef_resource_dir = Path::new("/usr/share").join(settings.product_name());
    // TODO: We probably want this in a shared and versioned location.
    let cef_temp_dir = package_dir.join("cef_temp");
    fs::create_dir_all(&cef_temp_dir).fs_context(
      "Failed to create temporary cef directory",
      cef_temp_dir.clone(),
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
      // sandbox - may need to be behind a setting?
      "chrome-sandbox",
    ];

    for f in cef_files {
      let temp_file = cef_temp_dir.join(f);
      fs::copy(cef_path.join(f), &temp_file)?;
      if f.ends_with(".so") {
        // since libcef.so is 1.5GB unstripped we will error out if strip fails.
        Command::new("strip").arg(&temp_file).output_ok()?;
      }
      let mut fileopts = FileOptions::new(cef_resource_dir.join(f).to_string_lossy());
      if f == "chrome-sandbox" {
        fileopts = fileopts.mode(0o104755);
      }
      builder.with_file(temp_file, fileopts).unwrap();
    }
    let locales = [
      "en-US.pak",
      "en-US_FEMININE.pak",
      "en-US_MASCULINE.pak",
      "en-US_NEUTER.pak",
    ];

    let cef_path = cef_path.join("locales");
    let cef_resource_dir = cef_resource_dir.join("locales");

    for f in locales {
      builder.with_file(
        cef_path.join(f),
        FileOptions::new(cef_resource_dir.join(f).to_string_lossy()),
      )?;
    }
  }

  // Add Desktop entry file
  let (desktop_src_path, desktop_dest_path) =
    freedesktop::generate_desktop_file(settings, &settings.rpm().desktop_template, &package_dir)?;
  builder.with_file(
    desktop_src_path,
    FileOptions::new(desktop_dest_path.to_string_lossy()),
  )?;

  // Add icons
  for (icon, src) in &freedesktop::list_icon_files(settings, &PathBuf::from("/"))? {
    builder.with_file(src, FileOptions::new(icon.path.to_string_lossy()))?;
  }

  // Add custom files
  for (rpm_path, src_path) in settings.rpm().files.iter() {
    if src_path.is_file() {
      builder.with_file(src_path, FileOptions::new(rpm_path.to_string_lossy()))?;
    } else {
      for entry in walkdir::WalkDir::new(src_path) {
        let entry_path = entry?.into_path();
        if entry_path.is_file() {
          let dest_path = rpm_path.join(entry_path.strip_prefix(src_path).unwrap());
          builder.with_file(&entry_path, FileOptions::new(dest_path.to_string_lossy()))?;
        }
      }
    }
  }

  log::info!(action = "Bundling"; "Creating .rpm file...");

  let pkg = if let Ok(raw_secret_key) = env::var("TAURI_SIGNING_RPM_KEY") {
    let mut signer = pgp::Signer::from_asc(&raw_secret_key)?;
    if let Ok(passphrase) = env::var("TAURI_SIGNING_RPM_KEY_PASSPHRASE") {
      signer = signer.with_key_passphrase(passphrase);
    }
    builder.build_and_sign(signer)?
  } else {
    builder.build()?
  };

  let mut f = fs::File::create(&package_path)?;
  pkg.write(&mut f)?;
  Ok(vec![package_path])
}
