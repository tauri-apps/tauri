// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;

use crate::{
  error::{Context, Error},
  helpers::cross_command,
};
use std::{collections::HashMap, fmt::Display, path::Path, process::Command};

pub fn manager_version(package_manager: &str) -> Option<String> {
  cross_command(package_manager)
    .arg("-v")
    .output()
    .map(|o| {
      if o.status.success() {
        let v = String::from_utf8_lossy(o.stdout.as_slice()).to_string();
        Some(v.split('\n').next().unwrap().to_string())
      } else {
        None
      }
    })
    .ok()
    .unwrap_or_default()
}

fn detect_yarn_or_berry() -> PackageManager {
  if manager_version("yarn")
    .map(|v| v.chars().next().map(|c| c > '1').unwrap_or_default())
    .unwrap_or(false)
  {
    PackageManager::YarnBerry
  } else {
    PackageManager::Yarn
  }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PackageManager {
  Npm,
  Pnpm,
  Yarn,
  YarnBerry,
  Bun,
  Deno,
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
        PackageManager::Deno => "deno",
      }
    )
  }
}

impl PackageManager {
  /// Detects package manager from the given directory, falls back to [`PackageManager::Npm`].
  pub fn from_project<P: AsRef<Path>>(path: P) -> Self {
    Self::all_from_project(path)
      .first()
      .copied()
      .unwrap_or(Self::Npm)
  }

  /// Detects package manager from the `npm_config_user_agent` environment variable
  fn from_environment_variable() -> Option<Self> {
    let npm_config_user_agent = std::env::var("npm_config_user_agent").ok()?;
    match npm_config_user_agent {
      user_agent if user_agent.starts_with("pnpm/") => Some(Self::Pnpm),
      user_agent if user_agent.starts_with("deno/") => Some(Self::Deno),
      user_agent if user_agent.starts_with("bun/") => Some(Self::Bun),
      user_agent if user_agent.starts_with("yarn/") => Some(detect_yarn_or_berry()),
      user_agent if user_agent.starts_with("npm/") => Some(Self::Npm),
      _ => None,
    }
  }

  /// Detects all possible package managers from the given directory.
  pub fn all_from_project<P: AsRef<Path>>(path: P) -> Vec<Self> {
    if let Some(from_env) = Self::from_environment_variable() {
      return vec![from_env];
    }

    let mut found = Vec::new();

    if let Ok(entries) = std::fs::read_dir(path) {
      for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().unwrap().to_string_lossy();
        match name.as_ref() {
          "package-lock.json" => found.push(PackageManager::Npm),
          "pnpm-lock.yaml" => found.push(PackageManager::Pnpm),
          "yarn.lock" => found.push(detect_yarn_or_berry()),
          "bun.lock" | "bun.lockb" => found.push(PackageManager::Bun),
          "deno.lock" => found.push(PackageManager::Deno),
          _ => (),
        }
      }
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
      PackageManager::Deno => cross_command("deno"),
    }
  }

  pub fn install<P: AsRef<Path>>(
    &self,
    dependencies: &[String],
    frontend_dir: P,
  ) -> crate::Result<()> {
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

    let mut command = self.cross_command();
    command.arg("add");

    match self {
      PackageManager::Deno => command.args(dependencies.iter().map(|d| format!("npm:{d}"))),
      _ => command.args(dependencies),
    };

    let status = command
      .current_dir(frontend_dir)
      .status()
      .map_err(|error| Error::CommandFailed {
        command: format!("failed to run {self}"),
        error,
      })?;

    if !status.success() {
      crate::error::bail!("Failed to install NPM {dependencies_str}");
    }

    Ok(())
  }

  pub fn remove<P: AsRef<Path>>(
    &self,
    dependencies: &[String],
    frontend_dir: P,
  ) -> crate::Result<()> {
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
      .current_dir(frontend_dir)
      .status()
      .map_err(|error| Error::CommandFailed {
        command: format!("failed to run {self}"),
        error,
      })?;

    if !status.success() {
      crate::error::bail!("Failed to remove NPM {dependencies_str}");
    }

    Ok(())
  }

  // TODO: Use `current_package_versions` as much as possible for better speed
  pub fn current_package_version<P: AsRef<Path>>(
    &self,
    name: &str,
    frontend_dir: P,
  ) -> crate::Result<Option<String>> {
    let (output, regex) = match self {
      PackageManager::Yarn => (
        cross_command("yarn")
          .args(["list", "--pattern"])
          .arg(name)
          .args(["--depth", "0"])
          .current_dir(frontend_dir)
          .output()
          .map_err(|error| Error::CommandFailed {
            command: "yarn list --pattern".to_string(),
            error,
          })?,
        None,
      ),
      PackageManager::YarnBerry => (
        cross_command("yarn")
          .arg("info")
          .arg(name)
          .arg("--json")
          .current_dir(frontend_dir)
          .output()
          .map_err(|error| Error::CommandFailed {
            command: "yarn info --json".to_string(),
            error,
          })?,
        Some(regex::Regex::new("\"Version\":\"([\\da-zA-Z\\-\\.]+)\"").unwrap()),
      ),
      PackageManager::Pnpm => (
        cross_command("pnpm")
          .arg("list")
          .arg(name)
          .args(["--parseable", "--depth", "0"])
          .current_dir(frontend_dir)
          .output()
          .map_err(|error| Error::CommandFailed {
            command: "pnpm list --parseable --depth 0".to_string(),
            error,
          })?,
        None,
      ),
      // Bun and Deno don't support `list` command
      PackageManager::Npm | PackageManager::Bun | PackageManager::Deno => (
        cross_command("npm")
          .arg("list")
          .arg(name)
          .args(["version", "--depth", "0"])
          .current_dir(frontend_dir)
          .output()
          .map_err(|error| Error::CommandFailed {
            command: "npm list --version --depth 0".to_string(),
            error,
          })?,
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

  pub fn current_package_versions(
    &self,
    packages: &[String],
    frontend_dir: &Path,
  ) -> crate::Result<HashMap<String, semver::Version>> {
    let output = match self {
      PackageManager::Yarn => return yarn_package_versions(packages, frontend_dir),
      PackageManager::YarnBerry => return yarn_berry_package_versions(packages, frontend_dir),
      PackageManager::Pnpm => cross_command("pnpm")
        .arg("list")
        .args(packages)
        .args(["--json", "--depth", "0"])
        .current_dir(frontend_dir)
        .output()
        .map_err(|error| Error::CommandFailed {
          command: "pnpm list --json --depth 0".to_string(),
          error,
        })?,
      // Bun and Deno don't support `list` command
      PackageManager::Npm | PackageManager::Bun | PackageManager::Deno => cross_command("npm")
        .arg("list")
        .args(packages)
        .args(["--json", "--depth", "0"])
        .current_dir(frontend_dir)
        .output()
        .map_err(|error| Error::CommandFailed {
          command: "npm list --json --depth 0".to_string(),
          error,
        })?,
    };

    let mut versions = HashMap::new();
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
      return Ok(versions);
    }

    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ListOutput {
      #[serde(default)]
      dependencies: HashMap<String, ListDependency>,
      #[serde(default)]
      dev_dependencies: HashMap<String, ListDependency>,
    }

    #[derive(Deserialize)]
    struct ListDependency {
      version: String,
    }

    let json = if matches!(self, PackageManager::Pnpm) {
      serde_json::from_str::<Vec<ListOutput>>(&stdout)
        .ok()
        .and_then(|out| out.into_iter().next())
        .context("failed to parse pnpm list")?
    } else {
      serde_json::from_str::<ListOutput>(&stdout).context("failed to parse npm list")?
    };
    for (package, dependency) in json.dependencies.into_iter().chain(json.dev_dependencies) {
      let version = dependency.version;
      if let Ok(version) = semver::Version::parse(&version) {
        versions.insert(package, version);
      } else {
        log::debug!("Failed to parse version `{version}` for NPM package `{package}`");
      }
    }
    Ok(versions)
  }
}

fn yarn_package_versions(
  packages: &[String],
  frontend_dir: &Path,
) -> crate::Result<HashMap<String, semver::Version>> {
  let output = cross_command("yarn")
    .arg("list")
    .args(packages)
    .args(["--json", "--depth", "0"])
    .current_dir(frontend_dir)
    .output()
    .map_err(|error| Error::CommandFailed {
      command: "yarn list --json --depth 0".to_string(),
      error,
    })?;

  let mut versions = HashMap::new();
  let stdout = String::from_utf8_lossy(&output.stdout);
  if !output.status.success() {
    return Ok(versions);
  }

  #[derive(Deserialize)]
  struct YarnListOutput {
    data: YarnListOutputData,
  }

  #[derive(Deserialize)]
  struct YarnListOutputData {
    trees: Vec<YarnListOutputDataTree>,
  }

  #[derive(Deserialize)]
  struct YarnListOutputDataTree {
    name: String,
  }

  for line in stdout.lines() {
    if let Ok(tree) = serde_json::from_str::<YarnListOutput>(line) {
      for tree in tree.data.trees {
        let Some((name, version)) = tree.name.rsplit_once('@') else {
          continue;
        };
        if let Ok(version) = semver::Version::parse(version) {
          versions.insert(name.to_owned(), version);
        } else {
          log::debug!("Failed to parse version `{version}` for NPM package `{name}`");
        }
      }
      return Ok(versions);
    }
  }

  Ok(versions)
}

fn yarn_berry_package_versions(
  packages: &[String],
  frontend_dir: &Path,
) -> crate::Result<HashMap<String, semver::Version>> {
  let output = cross_command("yarn")
    .args(["info", "--json"])
    .current_dir(frontend_dir)
    .output()
    .map_err(|error| Error::CommandFailed {
      command: "yarn info --json".to_string(),
      error,
    })?;

  let mut versions = HashMap::new();
  let stdout = String::from_utf8_lossy(&output.stdout);
  if !output.status.success() {
    return Ok(versions);
  }

  #[derive(Deserialize)]
  struct YarnBerryInfoOutput {
    value: String,
    children: YarnBerryInfoOutputChildren,
  }

  #[derive(Deserialize)]
  #[serde(rename_all = "PascalCase")]
  struct YarnBerryInfoOutputChildren {
    version: String,
  }

  for line in stdout.lines() {
    if let Ok(info) = serde_json::from_str::<YarnBerryInfoOutput>(line) {
      let Some((name, _)) = info.value.rsplit_once('@') else {
        continue;
      };
      if !packages.iter().any(|package| package == name) {
        continue;
      }
      let version = info.children.version;
      if let Ok(version) = semver::Version::parse(&version) {
        versions.insert(name.to_owned(), version);
      } else {
        log::debug!("Failed to parse version `{version}` for NPM package `{name}`");
      }
    }
  }

  Ok(versions)
}
