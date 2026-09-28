// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, path::PathBuf};

use clap::Parser;
use tauri_utils::acl::capability::{Capability, PermissionEntry};

use crate::{
  Result,
  acl::{FileFormat, split_comma_separated, validate_capability_identifier},
  error::ErrorExt,
  helpers::prompts,
};

#[derive(Debug, Parser)]
#[clap(about = "Create a new capability file")]
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
  let dirs = crate::helpers::app_paths::resolve_dirs();

  let identifier = match options.identifier {
    Some(i) => i,
    None => prompts::input("What's the capability identifier?", None, false, false)?.unwrap(),
  };
  validate_capability_identifier(&identifier)?;

  let description = match options.description {
    Some(d) => Some(d),
    None => prompts::input::<String>("What's the capability description?", None, false, true)?
      .filter(|d| !d.is_empty()),
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
        Some(split_comma_separated(&d).collect())
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
        Some(split_comma_separated(&p).collect())
      }
    })
    .unwrap_or_default(),
  };

  let permissions = permissions
    .into_iter()
    .map(|p| match p.clone().try_into() {
      Ok(identifier) => Ok(PermissionEntry::PermissionRef(identifier)),
      Err(e) => crate::error::bail!("invalid permission `{}`: {}", p, e),
    })
    .collect::<Result<Vec<_>>>()?;

  let capability = Capability {
    identifier,
    description: description.unwrap_or_default(),
    remote: None,
    local: true,
    windows,
    webviews: Vec::new(),
    permissions,
    platforms: None,
  };

  let path = match options.out {
    // the file may not exist yet, so it cannot be canonicalized
    Some(o) => o,
    None => {
      let capabilities_dir = dirs.tauri.join("capabilities");
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
      std::fs::remove_file(&path).fs_context("failed to remove capability file", path.clone())?;
    } else {
      crate::error::bail!(msg);
    }
  }

  if let Some(parent) = path.parent() {
    std::fs::create_dir_all(parent).fs_context(
      "failed to create capability directory",
      parent.to_path_buf(),
    )?;
  }

  std::fs::write(&path, options.format.serialize(&capability)?)
    .fs_context("failed to write capability file", path.clone())?;

  log::info!(action = "Created"; "capability at {}", dunce::simplified(&path).display());

  Ok(())
}
