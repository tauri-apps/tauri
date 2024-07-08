// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{ffi::OsStr, path::Path, process::Command};

use anyhow::Context;

#[derive(Debug, Clone, Copy)]
pub struct AddOptions<'a, I: IntoIterator<Item = S>, S: AsRef<OsStr>> {
  pub features: Option<I>,
  pub cwd: Option<&'a Path>,
}

impl<'a> Default for AddOptions<'a, std::slice::Iter<'a, &'a str>, &'a &'a str> {
  fn default() -> Self {
    Self {
      features: None,
      cwd: Default::default(),
    }
  }
}

pub fn add<I, S>(dependencies: &[String], options: AddOptions<I, S>) -> crate::Result<()>
where
  I: IntoIterator<Item = S>,
  S: AsRef<OsStr>,
{
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

  if let Some(features) = options.features {
    let mut features = features.into_iter().peekable();
    if features.peek().is_some() {
      cmd.arg("--features").args(features);
    }
  }

  if let Some(cwd) = options.cwd {
    cmd.current_dir(cwd);
  }

  let status = cmd.status().with_context(|| "failed to run cargo")?;

  if !status.success() {
    anyhow::bail!("Failed to install Cargo {dependencies_str}");
  }

  Ok(())
}

#[derive(Debug, Default, Clone, Copy)]
pub struct AddOneOptions<'a> {
  pub version: Option<&'a str>,
  pub rev: Option<&'a str>,
  pub tag: Option<&'a str>,
  pub branch: Option<&'a str>,
  pub cwd: Option<&'a Path>,
  pub target: Option<&'a str>,
}

pub fn add_one(crate_name: &str, options: AddOneOptions) -> crate::Result<()> {
  let mut cargo = Command::new("cargo");
  cargo.arg("add");

  if let Some(version) = options.version {
    cargo.arg(format!("{}@{}", crate_name, version));
  } else {
    cargo.arg(crate_name);

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

  log::info!("Installing Cargo dependency \"{}\"...", crate_name);
  let status = cargo.status().context("failed to run `cargo add`")?;
  if !status.success() {
    anyhow::bail!("Failed to install Cargo dependency");
  }

  Ok(())
}
