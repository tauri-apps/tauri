// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashSet, path::PathBuf};

use clap::Parser;

use crate::{
  helpers::{app_paths::tauri_dir, prompts},
  Result,
};

#[derive(serde::Serialize)]
struct Capability {
  identifier: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  description: Option<String>,
  windows: HashSet<String>,
  permissions: HashSet<String>,
}

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
  /// Use toml for the capability file.
  #[clap(long)]
  toml: bool,
  /// The output file.
  #[clap(short, long)]
  out: Option<PathBuf>,
}

pub fn command(options: Options) -> Result<()> {
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
    description,
    windows,
    permissions,
  };

  let path = match options.out {
    Some(o) => o.canonicalize()?,
    None => {
      let dir = tauri_dir();
      let capabilities_dir = dir.join("capabilities");
      let extension = if options.toml { "toml" } else { "conf.json" };
      capabilities_dir.join(format!("{}.{extension}", capability.identifier))
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

  let contents = if options.toml {
    toml_edit::ser::to_string_pretty(&capability)?
  } else {
    serde_json::to_string_pretty(&capability)?
  };
  std::fs::write(&path, contents)?;

  log::info!(action = "Created"; "capability at {}", dunce::simplified(&path).display());

  Ok(())
}
