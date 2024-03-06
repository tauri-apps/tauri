// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{helpers::cross_command, Result};
use std::{fmt::Display, path::Path, process::ExitStatus};

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
        }
      }
    }

    if !use_npm && !use_pnpm && !use_yarn {
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

    found
  }

  pub fn install(&self, dependencies: &[String]) -> Result<ExitStatus> {
    match self {
      PackageManager::Yarn => {
        let mut cmd = cross_command("yarn");
        cmd
          .arg("add")
          .args(dependencies)
          .status()
          .map_err(Into::into)
      }
      PackageManager::YarnBerry => {
        let mut cmd = cross_command("yarn");
        cmd
          .arg("add")
          .args(dependencies)
          .status()
          .map_err(Into::into)
      }
      PackageManager::Npm => {
        let mut cmd = cross_command("npm");
        cmd
          .arg("install")
          .args(dependencies)
          .status()
          .map_err(Into::into)
      }
      PackageManager::Pnpm => {
        let mut cmd = cross_command("pnpm");
        cmd
          .arg("install")
          .args(dependencies)
          .status()
          .map_err(Into::into)
      }
      PackageManager::Bun => {
        let mut cmd = cross_command("bun");
        cmd
          .arg("install")
          .args(dependencies)
          .status()
          .map_err(Into::into)
      }
    }
  }
}
