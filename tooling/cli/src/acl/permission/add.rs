// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use clap::Parser;

use crate::{
  helpers::{app_paths::tauri_dir_opt, prompts},
  Result,
};

#[derive(Clone)]
enum TomlOrJson {
  Toml(toml_edit::Document),
  Json(serde_json::Value),
}

impl TomlOrJson {
  fn identifier(&self) -> &str {
    match self {
      TomlOrJson::Toml(t) => t
        .get("identifier")
        .and_then(|k| k.as_str())
        .unwrap_or_default(),
      TomlOrJson::Json(j) => j
        .get("identifier")
        .and_then(|k| k.as_str())
        .unwrap_or_default(),
    }
  }

  fn insert_permission(&mut self, idenitifer: String) {
    match self {
      TomlOrJson::Toml(t) => {
        let permissions = t.entry("permissions").or_insert_with(|| {
          toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new()))
        });
        if let Some(permissions) = permissions.as_array_mut() {
          permissions.push(idenitifer)
        };
      }

      TomlOrJson::Json(j) => {
        if let Some(o) = j.as_object_mut() {
          let permissions = o
            .entry("permissions")
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
          if let Some(permissions) = permissions.as_array_mut() {
            permissions.push(serde_json::Value::String(idenitifer))
          };
        }
      }
    };
  }

  fn to_string(&self) -> Result<String> {
    Ok(match self {
      TomlOrJson::Toml(t) => t.to_string(),
      TomlOrJson::Json(j) => serde_json::to_string_pretty(&j)?,
    })
  }
}

fn capability_from_path<P: AsRef<Path>>(path: P) -> Option<TomlOrJson> {
  match path.as_ref().extension().and_then(|o| o.to_str()) {
    Some("toml") => std::fs::read_to_string(&path)
      .ok()
      .and_then(|c| c.parse::<toml_edit::Document>().ok())
      .map(TomlOrJson::Toml),
    Some("json") => std::fs::read(&path)
      .ok()
      .and_then(|c| serde_json::from_slice::<serde_json::Value>(&c).ok())
      .map(TomlOrJson::Json),
    _ => None,
  }
}

#[derive(Debug, Parser)]
#[clap(about = "Add a permission to capabilities")]
pub struct Options {
  /// Permission to add.
  pub identifier: String,
  /// Capability to add the permission to.
  pub capability: Option<String>,
}

pub fn command(options: Options) -> Result<()> {
  let dir = match tauri_dir_opt() {
    Some(t) => t,
    None => std::env::current_dir()?,
  };

  let capabilities_dir = dir.join("capabilities");
  if !capabilities_dir.exists() {
    anyhow::bail!(
      "Couldn't find capabilities directory at {}",
      dunce::simplified(&capabilities_dir).display()
    );
  }

  let capabilities = std::fs::read_dir(&capabilities_dir)?
    .flatten()
    .filter(|e| e.file_type().map(|e| e.is_file()).unwrap_or_default())
    .filter_map(|e| {
      let path = e.path();
      capability_from_path(&path).and_then(|capability| match &options.capability {
        Some(c) => (c == capability.identifier()).then_some((capability, path)),
        None => Some((capability, path)),
      })
    })
    .collect::<Vec<_>>();

  let mut capabilities = if capabilities.len() > 1 {
    let selections = prompts::multiselect(
      &format!(
        "Choose which capabilities to add the permission `{}` to:",
        options.identifier
      ),
      capabilities
        .iter()
        .map(|(c, p)| {
          let id = c.identifier();
          if id.is_empty() {
            dunce::simplified(p).to_str().unwrap_or_default()
          } else {
            id
          }
        })
        .collect::<Vec<_>>()
        .as_slice(),
      None,
    )?;
    selections
      .into_iter()
      .map(|idx| capabilities[idx].clone())
      .collect()
  } else {
    capabilities
  };

  for (capability, path) in &mut capabilities {
    capability.insert_permission(options.identifier.clone());
    std::fs::write(&*path, capability.to_string()?)?;
    log::info!(action = "Added"; "permission `{}` to `{}` at {}", options.identifier, capability.identifier(), dunce::simplified(path).display());
  }

  Ok(())
}
