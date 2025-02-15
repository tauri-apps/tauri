// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

use clap::Parser;

use crate::{
  helpers::{app_paths::resolve_tauri_dir, prompts},
  Result,
};

#[derive(Clone)]
enum TomlOrJson {
  Toml(toml_edit::DocumentMut),
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

  fn platforms(&self) -> Option<Vec<&str>> {
    match self {
      TomlOrJson::Toml(t) => t.get("platforms").and_then(|k| {
        k.as_array()
          .and_then(|array| array.iter().map(|v| v.as_str()).collect())
      }),
      TomlOrJson::Json(j) => j.get("platforms").and_then(|k| {
        if let Some(array) = k.as_array() {
          let mut items = Vec::new();
          for item in array {
            if let Some(s) = item.as_str() {
              items.push(s);
            }
          }
          Some(items)
        } else {
          None
        }
      }),
    }
  }

  fn insert_permission(&mut self, identifier: String) {
    match self {
      TomlOrJson::Toml(t) => {
        let permissions = t.entry("permissions").or_insert_with(|| {
          toml_edit::Item::Value(toml_edit::Value::Array(toml_edit::Array::new()))
        });
        if let Some(permissions) = permissions.as_array_mut() {
          permissions.push(identifier)
        };
      }

      TomlOrJson::Json(j) => {
        if let Some(o) = j.as_object_mut() {
          let permissions = o
            .entry("permissions")
            .or_insert_with(|| serde_json::Value::Array(Vec::new()));
          if let Some(permissions) = permissions.as_array_mut() {
            permissions.push(serde_json::Value::String(identifier))
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
      .and_then(|c| c.parse::<toml_edit::DocumentMut>().ok())
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
  let dir = match resolve_tauri_dir() {
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

  let known_plugins = crate::helpers::plugins::known_plugins();
  let known_plugin = options
    .identifier
    .split_once(':')
    .and_then(|(plugin, _permission)| known_plugins.get(&plugin));

  let capabilities_iter = std::fs::read_dir(&capabilities_dir)?
    .flatten()
    .filter(|e| e.file_type().map(|e| e.is_file()).unwrap_or_default())
    .filter_map(|e| {
      let path = e.path();
      capability_from_path(&path).and_then(|capability| match &options.capability {
        Some(c) => (c == capability.identifier()).then_some((capability, path)),
        None => Some((capability, path)),
      })
    });

  let (desktop_only, mobile_only) = known_plugin
    .map(|p| (p.desktop_only, p.mobile_only))
    .unwrap_or_default();

  let expected_capability_config = if desktop_only {
    Some((
      vec![
        tauri_utils::platform::Target::MacOS.to_string(),
        tauri_utils::platform::Target::Windows.to_string(),
        tauri_utils::platform::Target::Linux.to_string(),
      ],
      "desktop",
    ))
  } else if mobile_only {
    Some((
      vec![
        tauri_utils::platform::Target::Android.to_string(),
        tauri_utils::platform::Target::Ios.to_string(),
      ],
      "mobile",
    ))
  } else {
    None
  };

  let capabilities = if let Some((expected_platforms, target_name)) = expected_capability_config {
    let mut capabilities = capabilities_iter
      .filter(|(capability, _path)| {
        capability.platforms().is_some_and(|platforms| {
          // all platforms must be in the expected platforms list
          platforms
            .iter()
            .all(|p| expected_platforms.contains(&p.to_string()))
        })
      })
      .collect::<Vec<_>>();

    if capabilities.is_empty() {
      let identifier = format!("{target_name}-capability");
      let capability_path = capabilities_dir.join(target_name).with_extension("json");
      log::info!(
        "Capability matching platforms {expected_platforms:?} not found, creating {}",
        capability_path.display()
      );
      capabilities.push((
        TomlOrJson::Json(serde_json::json!({
          "identifier": identifier,
          "platforms": expected_platforms,
          "windows": ["main"]
        })),
        capability_path,
      ));
    }

    capabilities
  } else {
    capabilities_iter.collect::<Vec<_>>()
  };

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

    if selections.is_empty() {
      anyhow::bail!("You did not select any capabilities to update");
    }

    selections
      .into_iter()
      .map(|idx| capabilities[idx].clone())
      .collect()
  } else {
    capabilities
  };

  if capabilities.is_empty() {
    anyhow::bail!("Could not find a capability to update");
  }

  for (capability, path) in &mut capabilities {
    capability.insert_permission(options.identifier.clone());
    std::fs::write(&*path, capability.to_string()?)?;
    log::info!(action = "Added"; "permission `{}` to `{}` at {}", options.identifier, capability.identifier(), dunce::simplified(path).display());
  }

  Ok(())
}
