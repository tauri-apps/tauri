// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::{Path, PathBuf};

use clap::Parser;

use crate::{
  Result,
  error::{Context, ErrorExt},
  helpers::{app_paths::resolve_tauri_dir, prompts},
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

  fn has_permission(&self, identifier: &str) -> bool {
    (|| {
      Some(match self {
        TomlOrJson::Toml(t) => t
          .get("permissions")?
          .as_array()?
          .iter()
          .any(|value| value.as_str() == Some(identifier)),

        TomlOrJson::Json(j) => j
          .as_object()?
          .get("permissions")?
          .as_array()?
          .iter()
          .any(|value| value.as_str() == Some(identifier)),
      })
    })()
    .unwrap_or_default()
  }

  fn to_string(&self) -> Result<String> {
    Ok(match self {
      TomlOrJson::Toml(t) => t.to_string(),
      TomlOrJson::Json(j) => {
        serde_json::to_string_pretty(&j).context("failed to serialize JSON")?
      }
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

/// Finds an identifier and a file path for a new `target_name` capability
/// that do not collide with an existing capability, so no user file is overwritten.
fn new_capability_name(
  capabilities_dir: &Path,
  target_name: &str,
  existing_identifiers: &[&str],
) -> (String, PathBuf) {
  (1..)
    .map(|n| {
      let suffix = if n == 1 {
        String::new()
      } else {
        format!("-{n}")
      };
      (
        format!("{target_name}-capability{suffix}"),
        capabilities_dir.join(format!("{target_name}{suffix}.json")),
      )
    })
    .find(|(identifier, path)| {
      !path.exists() && !existing_identifiers.contains(&identifier.as_str())
    })
    .expect("unbounded iterator always finds a name")
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
    None => std::env::current_dir().context("failed to resolve current directory")?,
  };

  let capabilities_dir = dir.join("capabilities");
  if !capabilities_dir.exists() {
    crate::error::bail!(
      "Couldn't find capabilities directory at {}",
      dunce::simplified(&capabilities_dir).display()
    );
  }

  let known_plugins = crate::helpers::plugins::known_plugins();
  let known_plugin = options
    .identifier
    .split_once(':')
    .and_then(|(plugin, _permission)| known_plugins.get(&plugin));

  let all_capabilities = std::fs::read_dir(&capabilities_dir)
    .fs_context(
      "failed to read capabilities directory",
      capabilities_dir.clone(),
    )?
    .flatten()
    .filter(|e| e.file_type().map(|e| e.is_file()).unwrap_or_default())
    .filter_map(|e| {
      let path = e.path();
      capability_from_path(&path).map(|capability| (capability, path))
    })
    .collect::<Vec<_>>();

  let capabilities_iter = all_capabilities
    .iter()
    .filter(|(capability, _path)| match &options.capability {
      Some(c) => c == capability.identifier(),
      None => true,
    })
    .cloned();

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

  let capabilities = if let Some(requested) = &options.capability {
    // the user asked for this capability explicitly, so use it even if its platforms do not match
    let capabilities = capabilities_iter.collect::<Vec<_>>();
    if capabilities.is_empty() {
      crate::error::bail!("Could not find capability `{}`", requested);
    }
    if let Some((expected_platforms, _target_name)) = &expected_capability_config {
      for (capability, path) in &capabilities {
        let platforms_match = capability.platforms().is_some_and(|platforms| {
          platforms
            .iter()
            .all(|p| expected_platforms.contains(&p.to_string()))
        });
        if !platforms_match {
          log::warn!(
            "Capability `{requested}` at {} is not restricted to the platforms {expected_platforms:?} that `{}` supports",
            dunce::simplified(path).display(),
            options.identifier
          );
        }
      }
    }
    capabilities
  } else if let Some((expected_platforms, target_name)) = expected_capability_config {
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
      let existing_identifiers = all_capabilities
        .iter()
        .map(|(capability, _path)| capability.identifier())
        .collect::<Vec<_>>();
      let (identifier, capability_path) =
        new_capability_name(&capabilities_dir, target_name, &existing_identifiers);
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
      crate::error::bail!("You did not select any capabilities to update");
    }

    selections
      .into_iter()
      .map(|idx| capabilities[idx].clone())
      .collect()
  } else {
    capabilities
  };

  if capabilities.is_empty() {
    crate::error::bail!("Could not find a capability to update");
  }

  for (capability, path) in &mut capabilities {
    if capability.has_permission(&options.identifier) {
      log::info!(
        "Permission `{}` already found in `{}` at {}",
        options.identifier,
        capability.identifier(),
        dunce::simplified(path).display()
      );
    } else {
      capability.insert_permission(options.identifier.clone());
      std::fs::write(&*path, capability.to_string()?)
        .fs_context("failed to write capability file", path.clone())?;
      log::info!(action = "Added"; "permission `{}` to `{}` at {}", options.identifier, capability.identifier(), dunce::simplified(path).display());
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::new_capability_name;

  #[test]
  fn new_capability_never_overwrites_an_existing_file() {
    let dir = tempfile::tempdir().unwrap();

    let (identifier, path) = new_capability_name(dir.path(), "desktop", &[]);
    assert_eq!(identifier, "desktop-capability");
    assert_eq!(path, dir.path().join("desktop.json"));

    std::fs::write(dir.path().join("desktop.json"), "{}").unwrap();
    let (identifier, path) = new_capability_name(dir.path(), "desktop", &[]);
    assert_eq!(identifier, "desktop-capability-2");
    assert_eq!(path, dir.path().join("desktop-2.json"));

    // an identifier used by another file is skipped too
    let (identifier, path) = new_capability_name(dir.path(), "desktop", &["desktop-capability-2"]);
    assert_eq!(identifier, "desktop-capability-3");
    assert_eq!(path, dir.path().join("desktop-3.json"));
  }
}
