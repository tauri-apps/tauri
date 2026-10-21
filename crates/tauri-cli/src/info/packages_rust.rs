// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ActionResult, SectionItem};
use crate::helpers::cargo_manifest::{
  CrateVersion, cargo_manifest_and_lock, crate_latest_version, crate_version,
};
use colored::Colorize;
use std::path::{Path, PathBuf};

pub fn items(frontend_dir: Option<&PathBuf>, tauri_dir: Option<&Path>) -> Vec<SectionItem> {
  let mut items = Vec::new();

  if (tauri_dir.is_some() || frontend_dir.is_some())
    && let Some(tauri_dir) = tauri_dir
  {
    let (manifest, lock) = cargo_manifest_and_lock(tauri_dir);
    for dep in [
      "tauri",
      "tauri-build",
      "tauri-runtime-wry",
      "tauri-runtime-cef",
      "wry",
      "tao",
      "cef",
    ] {
      let crate_version = crate_version(tauri_dir, manifest.as_ref(), lock.as_ref(), dep);
      let item = rust_section_item(dep, crate_version);
      items.push(item);
    }
  }

  let tauri_cli_rust_item = SectionItem::new().action(|| {
    std::process::Command::new("cargo")
      .arg("tauri")
      .arg("-V")
      .output()
      .ok()
      .map(|o| {
        if o.status.success() {
          let out = String::from_utf8_lossy(o.stdout.as_slice());
          let (package, version) = out.split_once(' ').unwrap_or_default();
          let version = version.strip_suffix('\n').unwrap_or(version);
          let version_suffix = semver::Version::parse(version)
            .ok()
            .and_then(|current_version| {
              latest_version(package).filter(|latest_version| &current_version < latest_version)
            })
            .map(|latest_version| outdated_suffix(&latest_version))
            .unwrap_or_default();
          format!("{package} 🦀: {version}{version_suffix}").into()
        } else {
          ActionResult::None
        }
      })
      .unwrap_or_default()
  });
  items.push(tauri_cli_rust_item);

  items
}

pub fn rust_section_item(dep: &str, crate_version: CrateVersion) -> SectionItem {
  let version = crate_version
    .version
    .as_ref()
    .and_then(|v| semver::Version::parse(v).ok());

  let version_suffix = version
    .and_then(|version| latest_version(dep).filter(|target_version| &version < target_version))
    .map(|target_version| outdated_suffix(&target_version));

  SectionItem::new().description(format!(
    "{} {}: {}{}",
    dep,
    "🦀",
    crate_version,
    version_suffix
      .map(|s| format!(",{s}"))
      .unwrap_or_else(|| "".into())
  ))
}

fn latest_version(name: &str) -> Option<semver::Version> {
  match crate_latest_version(name) {
    Ok(version) => version,
    Err(error) => {
      log::warn!("Failed to check the latest version of `{name}`: {error}");
      None
    }
  }
}

fn outdated_suffix(latest_version: &semver::Version) -> String {
  format!(
    " ({}, latest: {})",
    "outdated".yellow(),
    latest_version.to_string().green()
  )
}
