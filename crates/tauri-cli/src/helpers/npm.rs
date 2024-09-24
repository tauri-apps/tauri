// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;

use crate::helpers::cross_command;
use std::{fmt::Display, path::Path, process::Command};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackageManager {
  Npm,
  Pnpm,
  Yarn,
  YarnBerry,
  Bun,
}

impl Display for PackageManager {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(
      f,
      "{}",
      match self {
        PackageManager::Npm => "npm",
        PackageManager::Pnpm => "pnpm",
        PackageManager::Yarn => "yarn",
        PackageManager::YarnBerry => "yarn berry",
        PackageManager::Bun => "bun",
      }
    )
  }
}

impl PackageManager {
  pub fn from_project<P: AsRef<Path>>(path: P) -> Vec<Self> {
    let mut use_npm = false;
    let mut use_pnpm = false;
    let mut use_yarn = false;
    let mut use_bun = false;

    if let Ok(entries) = std::fs::read_dir(path) {
      for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        if name.as_ref() == "package-lock.json" {
          use_npm = true;
        } else if name.as_ref() == "pnpm-lock.yaml" {
          use_pnpm = true;
        } else if name.as_ref() == "yarn.lock" {
          use_yarn = true;
        } else if name.as_ref() == "bun.lockb" {
          use_bun = true;
        }
      }
    }

    if !use_npm && !use_pnpm && !use_yarn && !use_bun {
      return Vec::new();
    }

    let mut found = Vec::new();

    if use_npm {
      found.push(PackageManager::Npm);
    }
    if use_pnpm {
      found.push(PackageManager::Pnpm);
    }
    if use_yarn {
      found.push(PackageManager::Yarn);
    }
    if use_bun {
      found.push(PackageManager::Bun);
    }

    found
  }

  fn cross_command(&self) -> Command {
    match self {
      PackageManager::Yarn => cross_command("yarn"),
      PackageManager::YarnBerry => cross_command("yarn"),
      PackageManager::Npm => cross_command("npm"),
      PackageManager::Pnpm => cross_command("pnpm"),
      PackageManager::Bun => cross_command("bun"),
    }
  }

  pub fn install<P: AsRef<Path>>(&self, dependencies: &[String], app_dir: P) -> crate::Result<()> {
    let dependencies_str = if dependencies.len() > 1 {
      "dependencies"
    } else {
      "dependency"
    };
    log::info!(
      "Installing NPM {dependencies_str} {}...",
      dependencies
        .iter()
        .map(|d| format!("\"{d}\""))
        .collect::<Vec<_>>()
        .join(", ")
    );

    let status = self
      .cross_command()
      .arg("add")
      .args(dependencies)
      .current_dir(app_dir)
      .status()
      .with_context(|| format!("failed to run {self}"))?;

    if !status.success() {
      anyhow::bail!("Failed to install NPM {dependencies_str}");
    }

    Ok(())
  }

  pub fn remove<P: AsRef<Path>>(&self, dependencies: &[String], app_dir: P) -> crate::Result<()> {
    let dependencies_str = if dependencies.len() > 1 {
      "dependencies"
    } else {
      "dependency"
    };
    log::info!(
      "Removing NPM {dependencies_str} {}...",
      dependencies
        .iter()
        .map(|d| format!("\"{d}\""))
        .collect::<Vec<_>>()
        .join(", ")
    );

    let status = self
      .cross_command()
      .arg(if *self == Self::Npm {
        "uninstall"
      } else {
        "remove"
      })
      .args(dependencies)
      .current_dir(app_dir)
      .status()
      .with_context(|| format!("failed to run {self}"))?;

    if !status.success() {
      anyhow::bail!("Failed to remove NPM {dependencies_str}");
    }

    Ok(())
  }

  pub fn current_package_version<P: AsRef<Path>>(
    &self,
    name: &str,
    app_dir: P,
  ) -> crate::Result<Option<String>> {
    let (output, regex) = match self {
      PackageManager::Yarn => (
        cross_command("yarn")
          .args(["list", "--pattern"])
          .arg(name)
          .args(["--depth", "0"])
          .current_dir(app_dir)
          .output()?,
        None,
      ),
      PackageManager::YarnBerry => (
        cross_command("yarn")
          .arg("info")
          .arg(name)
          .arg("--json")
          .current_dir(app_dir)
          .output()?,
        Some(regex::Regex::new("\"Version\":\"([\\da-zA-Z\\-\\.]+)\"").unwrap()),
      ),
      PackageManager::Npm => (
        cross_command("npm")
          .arg("list")
          .arg(name)
          .args(["version", "--depth", "0"])
          .current_dir(app_dir)
          .output()?,
        None,
      ),
      PackageManager::Pnpm => (
        cross_command("pnpm")
          .arg("list")
          .arg(name)
          .args(["--parseable", "--depth", "0"])
          .current_dir(app_dir)
          .output()?,
        None,
      ),
      // Bun doesn't support `list` command
      PackageManager::Bun => (
        cross_command("npm")
          .arg("list")
          .arg(name)
          .args(["version", "--depth", "0"])
          .current_dir(app_dir)
          .output()?,
        None,
      ),
    };
    if output.status.success() {
      let stdout = String::from_utf8_lossy(&output.stdout);
      let regex = regex.unwrap_or_else(|| regex::Regex::new("@(\\d[\\da-zA-Z\\-\\.]+)").unwrap());
      Ok(
        regex
          .captures_iter(&stdout)
          .last()
          .and_then(|cap| cap.get(1).map(|v| v.as_str().to_string())),
      )
    } else {
      Ok(None)
    }
  }
}
