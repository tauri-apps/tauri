// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{Settings, bundle::settings::Arch, error::ErrorExt};

use rpm::{self, Dependency, FileOptions, signature::pgp};
use std::{
  env, fs,
  path::{Path, PathBuf},
};
use tauri_utils::config::RpmCompression;

use super::freedesktop;

/// Converts a SemVer version string to the RPM `Version` tag format.
///
/// RPM reserves `-` as the separator between `Version` and `Release`, and the
/// `rpm` crate (since v0.20) refuses a `Version` that contains it, so it is
/// replaced with `~`.
fn to_rpm_version(version: &str) -> String {
  match semver::Version::parse(version) {
    Ok(v) if !v.pre.is_empty() || !v.build.is_empty() => {
      let mut rpm = format!("{}.{}.{}", v.major, v.minor, v.patch);
      if !v.pre.is_empty() {
        rpm.push('~');
        rpm.push_str(&v.pre.as_str().replace('-', "."));
      }
      if !v.build.is_empty() {
        rpm.push('+');
        rpm.push_str(&v.build.as_str().replace('-', "."));
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
    .unwrap_or(rpm::CompressionWithLevel::Gzip(6));

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
    builder.with_file(src, FileOptions::new(dest.to_string_lossy()))?;
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

  // Add resources
  if settings.resource_files().count() > 0 {
    let resource_dir = Path::new("/usr/lib").join(settings.product_name());
    builder.with_dir_entry(FileOptions::dir(resource_dir.to_string_lossy()).permissions(0o755))?;
    // Then add the resources files in that directory
    for resource in settings.resource_files().iter() {
      let resource = resource?;
      let dest = resource_dir.join(resource.target());
      builder.with_file(resource.path(), FileOptions::new(dest.to_string_lossy()))?;
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

#[cfg(test)]
mod tests {
  use super::to_rpm_version;

  #[test]
  fn plain_release_unchanged() {
    assert_eq!(to_rpm_version("1.0.0"), "1.0.0");
    assert_eq!(to_rpm_version("2.4.1"), "2.4.1");
  }

  #[test]
  fn semver_prerelease_uses_tilde() {
    assert_eq!(to_rpm_version("1.0.0-alpha"), "1.0.0~alpha");
    assert_eq!(to_rpm_version("1.0.0-beta.11"), "1.0.0~beta.11");
    assert_eq!(to_rpm_version("1.0.0-rc.1"), "1.0.0~rc.1");
    assert_eq!(
      to_rpm_version("0.1.11-nightly.20261019.2"),
      "0.1.11~nightly.20261019.2"
    );
  }

  #[test]
  fn hyphens_in_prerelease_are_normalized_to_dots() {
    assert_eq!(to_rpm_version("1.0.0-rc-1"), "1.0.0~rc.1");
    assert_eq!(
      to_rpm_version("1.0.0-alpha-beta-1"),
      to_rpm_version("1.0.0-alpha.beta.1")
    );
  }

  #[test]
  fn build_metadata_preserved() {
    assert_eq!(to_rpm_version("1.0.0+build.1"), "1.0.0+build.1");
    assert_eq!(to_rpm_version("1.0.0+build-1"), "1.0.0+build.1");
    assert_eq!(to_rpm_version("1.0.0-alpha+build.1"), "1.0.0~alpha+build.1");
  }

  #[test]
  fn non_semver_input_passes_through() {
    assert_eq!(to_rpm_version("1.0.0.0"), "1.0.0.0");
    assert_eq!(to_rpm_version("1.0"), "1.0");
    assert_eq!(to_rpm_version("1.0.0~beta"), "1.0.0~beta");
  }
}
