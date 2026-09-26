// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::SectionItem;
use super::VersionMetadata;
use colored::Colorize;
use serde::Deserialize;
use std::path::PathBuf;

use crate::error::Context;
use crate::{
  error::Error,
  helpers::{cross_command, npm::PackageManager},
};

#[derive(Deserialize)]
struct YarnVersionInfo {
  data: Vec<String>,
}

pub fn npm_latest_version(pm: &PackageManager, name: &str) -> crate::Result<Option<String>> {
  match pm {
    PackageManager::Yarn => {
      let mut cmd = cross_command("yarn");

      let output = cmd
        .arg("info")
        .arg(name)
        .args(["version", "--json"])
        .output()
        .map_err(|error| Error::CommandFailed {
          command: "yarn info --json".to_string(),
          error,
        })?;
      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let info: YarnVersionInfo =
          serde_json::from_str(&stdout).context("failed to parse yarn info")?;
        Ok(Some(info.data.last().unwrap().to_string()))
      } else {
        Ok(None)
      }
    }
    PackageManager::YarnBerry => {
      let mut cmd = cross_command("yarn");

      let output = cmd
        .arg("npm")
        .arg("info")
        .arg(name)
        .args(["--fields", "version", "--json"])
        .output()
        .map_err(|error| Error::CommandFailed {
          command: "yarn npm info --fields version --json".to_string(),
          error,
        })?;
      if output.status.success() {
        let info: crate::PackageJson = serde_json::from_reader(std::io::Cursor::new(output.stdout))
          .context("failed to parse yarn npm info")?;
        Ok(info.version)
      } else {
        Ok(None)
      }
    }
    // Bun and Deno don't support show command
    PackageManager::Npm | PackageManager::Deno | PackageManager::Bun => {
      let mut cmd = cross_command("npm");

      let output = cmd
        .arg("show")
        .arg(name)
        .arg("version")
        .output()
        .map_err(|error| Error::CommandFailed {
          command: "npm show --version".to_string(),
          error,
        })?;
      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Some(stdout.replace('\n', "")))
      } else {
        Ok(None)
      }
    }
    PackageManager::Pnpm => {
      let mut cmd = cross_command("pnpm");

      let output = cmd
        .arg("info")
        .arg(name)
        .arg("version")
        .output()
        .map_err(|error| Error::CommandFailed {
          command: "pnpm info --version".to_string(),
          error,
        })?;
      if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(Some(stdout.replace('\n', "")))
      } else {
        Ok(None)
      }
    }
  }
}

pub fn package_manager(frontend_dir: &PathBuf) -> PackageManager {
  let found = PackageManager::all_from_project(frontend_dir);

  if found.is_empty() {
    println!(
      "{}: no lock files found, defaulting to npm",
      "WARNING".yellow()
    );
    return PackageManager::Npm;
  }

  let pkg_manager = found[0];

  if found.len() > 1 {
    println!(
      "{}: Only one package manager should be used, but found {}.\n         Please remove unused package manager lock files, will use {} for now!",
      "WARNING".yellow(),
      found
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" and "),
      pkg_manager
    );
  }

  pkg_manager
}

pub fn items(
  frontend_dir: Option<&PathBuf>,
  package_manager: PackageManager,
  metadata: &VersionMetadata,
) -> Vec<SectionItem> {
  let mut items = Vec::new();
  if let Some(frontend_dir) = frontend_dir {
    for (package, version) in [
      ("@tauri-apps/api", None),
      ("@tauri-apps/cli", Some(metadata.js_cli.version.clone())),
    ] {
      let frontend_dir = frontend_dir.clone();
      let item = nodejs_section_item(package.into(), version, frontend_dir, package_manager);
      items.push(item);
    }
  }

  items
}

pub fn nodejs_section_item(
  package: String,
  version: Option<String>,
  frontend_dir: PathBuf,
  package_manager: PackageManager,
) -> SectionItem {
  SectionItem::new().action(move || {
    let version = version.clone().unwrap_or_else(|| {
      package_manager
        .current_package_version(&package, &frontend_dir)
        .unwrap_or_default()
        .unwrap_or_default()
    });

    let latest_ver = super::packages_nodejs::npm_latest_version(&package_manager, &package)
      .unwrap_or_default()
      .unwrap_or_default();

    if version.is_empty() {
      format!("{} {}: not installed!", package, " ⱼₛ".black().on_yellow())
    } else {
      format!(
        "{} {}: {}{}",
        package,
        " ⱼₛ".black().on_yellow(),
        version,
        outdated_suffix(&version, &latest_ver)
      )
    }
    .into()
  })
}

/// Formats the " (outdated, latest: X)" suffix. The installed version is scraped from package
/// manager output and may not be valid semver (e.g. `2.6` from a pnpm peer-suffixed store
/// directory), so unparseable versions produce no suffix instead of panicking.
fn outdated_suffix(version: &str, latest_ver: &str) -> String {
  if version.is_empty() || latest_ver.is_empty() {
    return String::new();
  }

  match (
    semver::Version::parse(version),
    semver::Version::parse(latest_ver),
  ) {
    (Ok(version), Ok(target_version)) if version < target_version => {
      format!(" ({}, latest: {})", "outdated".yellow(), latest_ver.green())
    }
    _ => String::new(),
  }
}

#[cfg(test)]
mod tests {
  use super::outdated_suffix;

  #[test]
  fn empty_versions_produce_no_suffix() {
    assert_eq!(outdated_suffix("", "1.0.0"), "");
    assert_eq!(outdated_suffix("1.0.0", ""), "");
    assert_eq!(outdated_suffix("", ""), "");
  }

  #[test]
  fn unparseable_version_does_not_panic() {
    // e.g. `@tauri-apps+plugin-http@2.6_0468b058...` in the pnpm store yields `2.6`
    assert_eq!(outdated_suffix("2.6", "2.7.0"), "");
    assert_eq!(outdated_suffix("2.7.0", "2.6"), "");
    assert_eq!(outdated_suffix("not-a-version", "1.0.0"), "");
  }

  #[test]
  fn outdated_version_reports_suffix() {
    let suffix = outdated_suffix("1.0.0", "2.0.0");
    assert!(suffix.contains("outdated"), "{suffix}");
    assert!(suffix.contains("2.0.0"), "{suffix}");
  }

  #[test]
  fn up_to_date_or_newer_version_produces_no_suffix() {
    assert_eq!(outdated_suffix("1.0.0", "1.0.0"), "");
    assert_eq!(outdated_suffix("2.0.0", "1.0.0"), "");
  }
}
