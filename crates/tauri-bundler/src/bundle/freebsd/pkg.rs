// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  bundle::settings::Arch,
  error::{Context, ErrorExt},
  utils::{fs_utils, CommandExt},
  Settings,
};
use image::{codecs::png::PngDecoder, ImageDecoder};
use std::{
  ffi::OsStr,
  fs::{self, File},
  io::{BufReader, Write},
  os::unix::fs::PermissionsExt,
  path::{Path, PathBuf},
  process::Command,
};
use walkdir::WalkDir;

const DEFAULT_PORT_CATEGORY: &str = "x11";
const PREFIX: &str = "/usr/local";

/// Bundles the project as a FreeBSD pkg package.
pub fn bundle_project(settings: &Settings) -> crate::Result<Vec<PathBuf>> {
  let arch = freebsd_arch(settings.binary_arch())?;
  let port_name = sanitize_port_name(settings.product_name());
  let package_base_name = format!("{}-{}-{}", port_name, settings.version_string(), arch);
  let base_dir = settings.project_out_directory().join("bundle/pkg");
  let package_dir = base_dir.join(&package_base_name);
  fs_utils::remove_dir_all(&package_dir)?;

  let root_dir = package_dir.join("pkgroot");
  let prefix_dir = root_dir.join(PREFIX.trim_start_matches('/'));
  let metadata_dir = package_dir.join("metadata");
  let plist_path = package_dir.join("pkg-plist");
  let origin = format!("{DEFAULT_PORT_CATEGORY}/{port_name}");

  log::info!(action = "Bundling"; "{} as FreeBSD pkg", settings.product_name());

  generate_stage(settings, &prefix_dir).context("Failed to generate FreeBSD pkg stage")?;
  normalize_pkgroot_permissions(&root_dir)
    .context("Failed to normalize FreeBSD pkg file permissions")?;
  generate_pkg_plist(&plist_path, &prefix_dir).context("Failed to generate pkg plist")?;
  generate_pkg_metadata(settings, &port_name, &origin, &metadata_dir)
    .context("Failed to generate pkg metadata")?;

  fs::create_dir_all(&base_dir).fs_context("failed to create pkg output directory", &base_dir)?;
  let output_path = base_dir.join(format!("{port_name}-{}.pkg", settings.version_string()));
  let _ = fs::remove_file(&output_path);

  let status = Command::new("pkg")
    .args(["create", "-m"])
    .arg(&metadata_dir)
    .args(["-p"])
    .arg(&plist_path)
    .args(["-r"])
    .arg(&root_dir)
    .args(["-o"])
    .arg(&base_dir)
    .piped()
    .map_err(crate::Error::IoError)
    .context("failed to run pkg create")?;

  if !status.success() {
    return Err(crate::Error::GenericError(format!(
      "pkg create failed for {port_name}-{}",
      settings.version_string()
    )));
  }

  if !output_path.exists() {
    return Err(crate::Error::GenericError(format!(
      "pkg create completed but {} was not found",
      output_path.display()
    )));
  }

  Ok(vec![output_path])
}

fn normalize_pkgroot_permissions(root_dir: &Path) -> crate::Result<()> {
  for entry in WalkDir::new(root_dir).into_iter().filter_map(Result::ok) {
    let path = entry.path();
    if entry.file_type().is_dir() {
      fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    } else if entry.file_type().is_file() {
      let mode = if path
        .components()
        .any(|component| component.as_os_str() == OsStr::new("bin"))
      {
        0o755
      } else {
        0o644
      };
      fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
  }
  Ok(())
}

fn generate_stage(settings: &Settings, stage_dir: &Path) -> crate::Result<()> {
  let bin_dir = stage_dir.join("bin");
  for bin in settings.binaries() {
    let bin_path = settings.binary_path(bin);
    let dest = bin_dir.join(bin.name());
    fs_utils::copy_file(&bin_path, &dest)
      .with_context(|| format!("Failed to copy binary from {bin_path:?} to {dest:?}"))?;
  }

  settings
    .copy_binaries(&bin_dir)
    .context("Failed to copy external binaries")?;

  let resource_dir = stage_dir.join("lib").join(settings.product_name());
  settings
    .copy_resources(&resource_dir)
    .context("Failed to copy resource files")?;

  copy_icon_files(settings, stage_dir).context("Failed to copy icon files")?;
  generate_desktop_file(settings, stage_dir).context("Failed to create desktop file")?;

  Ok(())
}

fn generate_pkg_metadata(
  settings: &Settings,
  port_name: &str,
  origin: &str,
  metadata_dir: &Path,
) -> crate::Result<()> {
  fs::create_dir_all(metadata_dir).fs_context("failed to create pkg metadata", metadata_dir)?;

  let maintainer = settings
    .authors_comma_separated()
    .filter(|authors| authors.contains('@'))
    .or_else(|| settings.publisher().map(ToString::to_string))
    .filter(|publisher| publisher.contains('@'))
    .unwrap_or_else(|| "ports@FreeBSD.org".into());
  let comment = if settings.short_description().is_empty() {
    settings.product_name().to_string()
  } else {
    settings.short_description().to_string()
  };
  let description = settings
    .long_description()
    .filter(|description| !description.is_empty())
    .unwrap_or_else(|| {
      if settings.short_description().is_empty() {
        settings.product_name()
      } else {
        settings.short_description()
      }
    });
  let (abi, altabi) = pkg_abi()?;

  let mut manifest = fs_utils::create_file(&metadata_dir.join("+MANIFEST"))?;
  writeln!(manifest, "name: \"{}\"", ucl_escape(port_name))?;
  writeln!(
    manifest,
    "version: \"{}\"",
    ucl_escape(settings.version_string())
  )?;
  writeln!(manifest, "origin: \"{}\"", ucl_escape(origin))?;
  writeln!(manifest, "comment: \"{}\"", ucl_escape(&comment))?;
  writeln!(manifest, "maintainer: \"{}\"", ucl_escape(&maintainer))?;
  if let Some(homepage) = settings.homepage_url() {
    writeln!(manifest, "www: \"{}\"", ucl_escape(homepage))?;
  }
  writeln!(manifest, "abi: \"{}\"", ucl_escape(&abi))?;
  writeln!(manifest, "arch: \"{}\"", ucl_escape(&altabi))?;
  writeln!(manifest, "prefix: \"{PREFIX}\"")?;
  writeln!(manifest, "desc: <<EOD")?;
  writeln!(manifest, "{description}")?;
  writeln!(manifest, "EOD")?;
  if let Some((licenses, logic)) = settings.license().as_deref().and_then(freebsd_licenses) {
    writeln!(manifest, "licenselogic: \"{logic}\"")?;
    writeln!(manifest, "licenses: [")?;
    for license in licenses {
      writeln!(manifest, "    \"{}\"", ucl_escape(&license))?;
    }
    writeln!(manifest, "]")?;
  }
  writeln!(manifest, "deps: {{")?;
  for dependency in runtime_dependencies()? {
    let PackageDependency {
      name,
      origin,
      version,
    } = dependency;
    writeln!(manifest, "    {name}: {{")?;
    writeln!(manifest, "        origin: \"{origin}\"")?;
    writeln!(manifest, "        version: \"{}\"", ucl_escape(&version))?;
    writeln!(manifest, "    }}")?;
  }
  writeln!(manifest, "}}")?;

  Ok(())
}

fn generate_pkg_plist(path: &Path, stage_dir: &Path) -> crate::Result<()> {
  let mut entries = Vec::new();
  for entry in WalkDir::new(stage_dir).into_iter().filter_map(Result::ok) {
    if entry.file_type().is_file() {
      let rel = entry.path().strip_prefix(stage_dir)?.to_string_lossy();
      entries.push(rel.replace(std::path::MAIN_SEPARATOR, "/"));
    }
  }
  entries.sort();

  let mut file = fs_utils::create_file(path)?;
  for entry in entries {
    writeln!(file, "{entry}")?;
  }
  Ok(())
}

fn generate_desktop_file(settings: &Settings, stage_dir: &Path) -> crate::Result<()> {
  let bin_name = settings.main_binary_name()?;
  let desktop_path = stage_dir
    .join("share/applications")
    .join(format!("{}.desktop", settings.product_name()));
  let mut file = fs_utils::create_file(&desktop_path)?;

  writeln!(file, "[Desktop Entry]")?;
  writeln!(
    file,
    "Categories={}",
    settings
      .app_category()
      .map(|c| c.freedesktop_categories())
      .unwrap_or("")
  )?;
  if !settings.short_description().is_empty() {
    writeln!(file, "Comment={}", settings.short_description())?;
  }
  writeln!(file, "Exec={}", shell_word(bin_name))?;
  writeln!(file, "Icon={bin_name}")?;
  writeln!(file, "Name={}", settings.product_name())?;
  writeln!(file, "Terminal=false")?;
  writeln!(file, "Type=Application")?;
  if let Some(mime_type) = mime_types(settings) {
    writeln!(file, "MimeType={mime_type}")?;
  }

  Ok(())
}

fn copy_icon_files(settings: &Settings, stage_dir: &Path) -> crate::Result<()> {
  let main_binary_name = settings.main_binary_name()?;
  let base_dir = stage_dir.join("share/icons/hicolor");
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;
    if icon_path.extension() != Some(OsStr::new("png")) {
      continue;
    }
    let decoder = PngDecoder::new(BufReader::new(File::open(&icon_path)?))?;
    let (width, height) = decoder.dimensions();
    let scale = if crate::utils::is_retina(&icon_path) {
      "@2"
    } else {
      ""
    };
    let dest = base_dir.join(format!(
      "{width}x{height}{scale}/apps/{main_binary_name}.png"
    ));
    fs_utils::copy_file(&icon_path, &dest)?;
  }
  Ok(())
}

fn mime_types(settings: &Settings) -> Option<String> {
  let mut mime_types = Vec::new();
  if let Some(associations) = settings.file_associations() {
    mime_types.extend(
      associations
        .iter()
        .filter_map(|association| association.mime_type.clone()),
    );
  }
  if let Some(protocols) = settings.deep_link_protocols() {
    mime_types.extend(
      protocols
        .iter()
        .flat_map(|protocol| &protocol.schemes)
        .map(|scheme| format!("x-scheme-handler/{scheme}")),
    );
  }
  (!mime_types.is_empty()).then(|| mime_types.join(";"))
}

fn freebsd_arch(arch: Arch) -> crate::Result<&'static str> {
  match arch {
    Arch::X86_64 => Ok("amd64"),
    Arch::X86 => Ok("i386"),
    Arch::AArch64 => Ok("arm64"),
    Arch::Armhf => Ok("armv7"),
    Arch::Riscv64 => Ok("riscv64"),
    target => Err(crate::Error::ArchError(format!(
      "Unsupported FreeBSD architecture: {target:?}"
    ))),
  }
}

fn sanitize_port_name(name: &str) -> String {
  let mut sanitized = String::with_capacity(name.len());
  for ch in name.chars().flat_map(char::to_lowercase) {
    if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.') {
      sanitized.push(ch);
    } else if !sanitized.ends_with('-') {
      sanitized.push('-');
    }
  }
  sanitized.trim_matches('-').to_string()
}

fn shell_word(word: &str) -> String {
  if word.contains(' ') {
    format!("\"{}\"", word.replace('"', "\\\""))
  } else {
    word.to_string()
  }
}

fn pkg_abi() -> crate::Result<(String, String)> {
  let abi = pkg_config("ABI")?;
  let altabi = pkg_config("ALTABI")?;
  Ok((abi, altabi))
}

fn pkg_config(key: &str) -> crate::Result<String> {
  let output = Command::new("pkg")
    .args(["config", key])
    .output()
    .map_err(crate::Error::IoError)
    .with_context(|| format!("failed to run pkg config {key}"))?;
  if !output.status.success() {
    return Err(crate::Error::GenericError(format!(
      "pkg config {key} failed"
    )));
  }
  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

struct PackageDependency {
  name: &'static str,
  origin: &'static str,
  version: String,
}

fn runtime_dependencies() -> crate::Result<Vec<PackageDependency>> {
  [
    ("dbus", "devel/dbus"),
    ("gtk3", "x11-toolkits/gtk30"),
    ("webkit2-gtk_41", "www/webkit2-gtk"),
  ]
  .into_iter()
  .map(|(name, origin)| {
    Ok(PackageDependency {
      name,
      origin,
      version: package_version(name)?,
    })
  })
  .collect()
}

fn package_version(name: &str) -> crate::Result<String> {
  let output = Command::new("pkg")
    .args(["query", "%v", name])
    .output()
    .map_err(crate::Error::IoError)
    .with_context(|| format!("failed to query FreeBSD package {name}"))?;

  if !output.status.success() {
    return Err(crate::Error::GenericError(format!(
      "FreeBSD package dependency {name} is not installed"
    )));
  }

  Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn freebsd_licenses(license: &str) -> Option<(Vec<String>, &'static str)> {
  let license = license.trim();
  if license.is_empty() {
    return None;
  }

  let (separator, logic) = if license.contains(" OR ") {
    (" OR ", "or")
  } else if license.contains(" AND ") {
    (" AND ", "and")
  } else {
    (" ", "single")
  };

  let licenses = license
    .split(separator)
    .filter_map(freebsd_license_token)
    .map(ToString::to_string)
    .collect::<Vec<_>>();

  (!licenses.is_empty()).then_some((licenses, logic))
}

fn freebsd_license_token(license: &str) -> Option<&'static str> {
  match license.trim_matches(|c| c == '(' || c == ')').trim() {
    "Apache-2.0" => Some("APACHE20"),
    "BSD-2-Clause" => Some("BSD2CLAUSE"),
    "BSD-3-Clause" => Some("BSD3CLAUSE"),
    "GPL-2.0" | "GPL-2.0-only" => Some("GPLv2"),
    "GPL-3.0" | "GPL-3.0-only" => Some("GPLv3"),
    "LGPL-2.1" | "LGPL-2.1-only" => Some("LGPL21"),
    "LGPL-3.0" | "LGPL-3.0-only" => Some("LGPL3"),
    "MIT" => Some("MIT"),
    "MPL-2.0" => Some("MPL20"),
    _ => None,
  }
}

fn ucl_escape(value: &str) -> String {
  value.replace('\\', "\\\\").replace('"', "\\\"")
}
