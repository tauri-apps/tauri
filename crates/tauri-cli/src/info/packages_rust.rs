// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ActionResult, SectionItem};
use crate::helpers::cargo_manifest::{
  cargo_manifest_and_lock, crate_latest_version, crate_version, CrateVersion,
};
use colored::Colorize;
use std::path::{Path, PathBuf};

pub fn items(frontend_dir: Option<&PathBuf>, tauri_dir: Option<&Path>) -> Vec<SectionItem> {
  let mut items = Vec::new();

  if tauri_dir.is_some() || frontend_dir.is_some() {
    if let Some(tauri_dir) = tauri_dir {
      let (manifest, lock) = cargo_manifest_and_lock(tauri_dir);
      for dep in ["tauri", "tauri-build", "wry", "tao"] {
        let crate_version = crate_version(tauri_dir, manifest.as_ref(), lock.as_ref(), dep);
        let item = rust_section_item(dep, crate_version);
        items.push(item);
      }
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
          let latest_version = crate_latest_version(package).unwrap_or_default();
          format!(
            "{package} 🦀: {version}{}",
            if !(version.is_empty() || latest_version.is_empty()) {
              let current_version = semver::Version::parse(version).unwrap();
              let target_version = semver::Version::parse(latest_version.as_str()).unwrap();

              if current_version < target_version {
                format!(
                  " ({}, latest: {})",
                  "outdated".yellow(),
                  latest_version.green()
                )
              } else {
                "".into()
              }
            } else {
              "".into()
            }
          )
          .into()
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

  let version_suffix = match (version, crate_latest_version(dep)) {
    (Some(version), Some(target_version)) => {
      let target_version = semver::Version::parse(&target_version).unwrap();
      if version < target_version {
        Some(format!(
          " ({}, latest: {})",
          "outdated".yellow(),
          target_version.to_string().green()
        ))
      } else {
        None
      }
    }
    _ => None,
  };

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
