// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use clap::Parser;

use crate::{
  acl::FileFormat,
  helpers::{app_paths::resolve_tauri_dir, prompts},
  Result,
};

use tauri_utils::acl::{manifest::PermissionFile, Commands, Permission};

#[derive(Debug, Parser)]
#[clap(about = "Create a new permission file")]
pub struct Options {
  /// Permission identifier.
  identifier: Option<String>,
  /// Permission description
  #[clap(long)]
  description: Option<String>,
  /// List of commands to allow
  #[clap(short, long, use_value_delimiter = true)]
  allow: Option<Vec<String>>,
  /// List of commands to deny
  #[clap(short, long, use_value_delimiter = true)]
  deny: Option<Vec<String>>,
  /// Output file format.
  #[clap(long, default_value_t = FileFormat::Json)]
  format: FileFormat,
  /// The output file.
  #[clap(short, long)]
  out: Option<PathBuf>,
}

pub fn command(options: Options) -> Result<()> {
  let identifier = match options.identifier {
    Some(i) => i,
    None => prompts::input("What's the permission identifier?", None, false, false)?.unwrap(),
  };

  let description = match options.description {
    Some(d) => Some(d),
    None => prompts::input::<String>("What's the permission description?", None, false, true)?
      .and_then(|d| if d.is_empty() { None } else { Some(d) }),
  };

  let allow: Vec<String> = options
    .allow
    .map(FromIterator::from_iter)
    .unwrap_or_default();
  let deny: Vec<String> = options
    .deny
    .map(FromIterator::from_iter)
    .unwrap_or_default();

  let permission = Permission {
    version: None,
    identifier,
    description,
    commands: Commands { allow, deny },
    scope: Default::default(),
    platforms: Default::default(),
  };

  let path = match options.out {
    Some(o) => o.canonicalize()?,
    None => {
      let dir = match resolve_tauri_dir() {
        Some(t) => t,
        None => std::env::current_dir()?,
      };
      let permissions_dir = dir.join("permissions");
      permissions_dir.join(format!(
        "{}.{}",
        permission.identifier,
        options.format.extension()
      ))
    }
  };

  if path.exists() {
    let msg = format!(
      "Permission already exists at {}",
      dunce::simplified(&path).display()
    );
    let overwrite = prompts::confirm(&format!("{msg}, overwrite?"), Some(false))?;
    if overwrite {
      std::fs::remove_file(&path)?;
    } else {
      anyhow::bail!(msg);
    }
  }

  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent)?;
  }

  std::fs::write(
    &path,
    options.format.serialize(&PermissionFile {
      default: None,
      set: Vec::new(),
      permission: vec![permission],
    })?,
  )?;

  log::info!(action = "Created"; "permission at {}", dunce::simplified(&path).display());

  Ok(())
}
