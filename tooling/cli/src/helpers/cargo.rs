// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{path::Path, process::Command};

use anyhow::Context;

#[derive(Debug, Default, Clone, Copy)]
pub struct CargoInstallOptions<'a> {
  pub name: &'a str,
  pub version: Option<&'a str>,
  pub rev: Option<&'a str>,
  pub tag: Option<&'a str>,
  pub branch: Option<&'a str>,
  pub cwd: Option<&'a std::path::Path>,
  pub target: Option<&'a str>,
}

pub fn install(dependencies: &[String], cwd: Option<&Path>) -> crate::Result<()> {
  let dependencies_str = if dependencies.len() > 1 {
    "dependencies"
  } else {
    "dependency"
  };
  log::info!(
    "Installing Cargo {dependencies_str} {}...",
    dependencies
      .iter()
      .map(|d| format!("\"{d}\""))
      .collect::<Vec<_>>()
      .join(", ")
  );

  let mut cmd = Command::new("cargo");
  cmd.arg("add").args(dependencies);

  if let Some(cwd) = cwd {
    cmd.current_dir(cwd);
  }

  let status = cmd.status().with_context(|| "failed to run cargo")?;

  if !status.success() {
    anyhow::bail!("Failed to install Cargo {dependencies_str}");
  }

  Ok(())
}

pub fn install_one(options: CargoInstallOptions) -> crate::Result<()> {
  let mut cargo = Command::new("cargo");
  cargo.arg("add");

  if let Some(version) = options.version {
    cargo.arg(format!("{}@{}", options.name, version));
  } else {
    cargo.arg(options.name);

    if options.tag.is_some() || options.rev.is_some() || options.branch.is_some() {
      cargo.args(["--git", "https://github.com/tauri-apps/plugins-workspace"]);
    }

    match (options.tag, options.rev, options.branch) {
      (Some(tag), None, None) => {
        cargo.args(["--tag", &tag]);
      }
      (None, Some(rev), None) => {
        cargo.args(["--rev", &rev]);
      }
      (None, None, Some(branch)) => {
        cargo.args(["--branch", &branch]);
      }
      (None, None, None) => {}
      _ => anyhow::bail!("Only one of --tag, --rev and --branch can be specified"),
    };
  }

  if let Some(target) = options.target {
    cargo.args(["--target", target]);
  }

  if let Some(cwd) = options.cwd {
    cargo.current_dir(cwd);
  }

  log::info!("Installing Cargo dependency \"{}\"...", options.name);
  let status = cargo.status().context("failed to run `cargo add`")?;
  if !status.success() {
    anyhow::bail!("Failed to install Cargo dependency");
  }

  Ok(())
}
