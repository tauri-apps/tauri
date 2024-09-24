// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use tauri_utils::acl::capability::{Capability, PermissionEntry};

use crate::{
  acl::FileFormat,
  helpers::{app_paths::tauri_dir, prompts},
  Result,
};

#[derive(Debug, Parser)]
#[clap(about = "Create a new permission file")]
pub struct Options {
  /// Capability identifier.
  identifier: Option<String>,
  /// Capability description
  #[clap(long)]
  description: Option<String>,
  /// Capability windows
  #[clap(long)]
  windows: Option<Vec<String>>,
  /// Capability permissions
  #[clap(long)]
  permission: Option<Vec<String>>,
  /// Output file format.
  #[clap(long, default_value_t = FileFormat::Json)]
  format: FileFormat,
  /// The output file.
  #[clap(short, long)]
  out: Option<PathBuf>,
}

pub fn command(options: Options) -> Result<()> {
  crate::helpers::app_paths::resolve();

  let identifier = match options.identifier {
    Some(i) => i,
    None => prompts::input("What's the capability identifier?", None, false, false)?.unwrap(),
  };

  let description = match options.description {
    Some(d) => Some(d),
    None => prompts::input::<String>("What's the capability description?", None, false, true)?
      .and_then(|d| if d.is_empty() { None } else { Some(d) }),
  };

  let windows = match options.windows.map(FromIterator::from_iter) {
    Some(w) => w,
    None => prompts::input::<String>(
      "Which windows should be affected by this? (comma separated)",
      Some("main".into()),
      false,
      false,
    )?
    .and_then(|d| {
      if d.is_empty() {
        None
      } else {
        Some(d.split(',').map(ToString::to_string).collect())
      }
    })
    .unwrap_or_default(),
  };

  let permissions: HashSet<String> = match options.permission.map(FromIterator::from_iter) {
    Some(p) => p,
    None => prompts::input::<String>(
      "What permissions to enable? (comma separated)",
      None,
      false,
      true,
    )?
    .and_then(|p| {
      if p.is_empty() {
        None
      } else {
        Some(p.split(',').map(ToString::to_string).collect())
      }
    })
    .unwrap_or_default(),
  };

  let capability = Capability {
    identifier,
    description: description.unwrap_or_default(),
    remote: None,
    local: true,
    windows,
    webviews: Vec::new(),
    permissions: permissions
      .into_iter()
      .map(|p| {
        PermissionEntry::PermissionRef(
          p.clone()
            .try_into()
            .unwrap_or_else(|_| panic!("invalid permission {}", p)),
        )
      })
      .collect(),
    platforms: None,
  };

  let path = match options.out {
    Some(o) => o.canonicalize()?,
    None => {
      let dir = tauri_dir();
      let capabilities_dir = dir.join("capabilities");
      capabilities_dir.join(format!(
        "{}.{}",
        capability.identifier,
        options.format.extension()
      ))
    }
  };

  if path.exists() {
    let msg = format!(
      "Capability already exists at {}",
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

  std::fs::write(&path, options.format.serialize(&capability)?)?;

  log::info!(action = "Created"; "capability at {}", dunce::simplified(&path).display());

  Ok(())
}
