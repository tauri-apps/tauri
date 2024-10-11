// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::Command;

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
        cargo.args(["--tag", tag]);
      }
      (None, Some(rev), None) => {
        cargo.args(["--rev", rev]);
      }
      (None, None, Some(branch)) => {
        cargo.args(["--branch", branch]);
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

#[derive(Debug, Default, Clone, Copy)]
pub struct CargoUninstallOptions<'a> {
  pub name: &'a str,
  pub cwd: Option<&'a std::path::Path>,
  pub target: Option<&'a str>,
}

pub fn uninstall_one(options: CargoUninstallOptions) -> crate::Result<()> {
  let mut cargo = Command::new("cargo");
  cargo.arg("remove");

  cargo.arg(options.name);

  if let Some(target) = options.target {
    cargo.args(["--target", target]);
  }

  if let Some(cwd) = options.cwd {
    cargo.current_dir(cwd);
  }

  log::info!("Uninstalling Cargo dependency \"{}\"...", options.name);
  let status = cargo.status().context("failed to run `cargo remove`")?;
  if !status.success() {
    anyhow::bail!("Failed to remove Cargo dependency");
  }

  Ok(())
}
