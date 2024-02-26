// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use clap::Parser;

use crate::{helpers::app_paths::tauri_dir, Result};
use colored::Colorize;
use tauri_utils::acl::plugin::Manifest;

use std::{collections::BTreeMap, fs::read_to_string};

#[derive(Debug, Parser)]
#[clap(about = "List permissions available to your application")]
pub struct Options {
  /// Name of the plugin to list permissions.
  plugin: Option<String>,
  /// Permission identifier filter.
  #[clap(short, long)]
  filter: Option<String>,
}

pub fn command(options: Options) -> Result<()> {
  let tauri_dir = tauri_dir();
  let plugin_manifests_path = tauri_dir
    .join("gen")
    .join("schemas")
    .join("plugin-manifests.json");

  if plugin_manifests_path.exists() {
    let plugin_manifest_json = read_to_string(&plugin_manifests_path)?;
    let acl = serde_json::from_str::<BTreeMap<String, Manifest>>(&plugin_manifest_json)?;

    for (plugin, manifest) in acl {
      if options
        .plugin
        .as_ref()
        .map(|p| p != &plugin)
        .unwrap_or_default()
      {
        continue;
      }

      let mut permissions = Vec::new();

      if let Some(default) = manifest.default_permission {
        if options
          .filter
          .as_ref()
          .map(|f| "default".contains(f))
          .unwrap_or(true)
        {
          permissions.push(format!(
            "{}:{}\n{}\nPermissions: {}",
            plugin.magenta(),
            "default".cyan(),
            default.description,
            default
              .permissions
              .iter()
              .map(|c| c.cyan().to_string())
              .collect::<Vec<_>>()
              .join(", ")
          ));
        }
      }

      for set in manifest.permission_sets.values() {
        if options
          .filter
          .as_ref()
          .map(|f| set.identifier.contains(f))
          .unwrap_or(true)
        {
          permissions.push(format!(
            "{}:{}\n{}\nPermissions: {}",
            plugin.magenta(),
            set.identifier.cyan(),
            set.description,
            set
              .permissions
              .iter()
              .map(|c| c.cyan().to_string())
              .collect::<Vec<_>>()
              .join(", ")
          ));
        }
      }

      for permission in manifest.permissions.into_values() {
        if options
          .filter
          .as_ref()
          .map(|f| permission.identifier.contains(f))
          .unwrap_or(true)
        {
          permissions.push(format!(
            "{}:{}{}{}{}",
            plugin.magenta(),
            permission.identifier.cyan(),
            permission
              .description
              .map(|d| format!("\n{d}"))
              .unwrap_or_default(),
            if permission.commands.allow.is_empty() {
              "".to_string()
            } else {
              format!(
                "\n{}: {}",
                "Allow commands".bold(),
                permission
                  .commands
                  .allow
                  .iter()
                  .map(|c| c.green().to_string())
                  .collect::<Vec<_>>()
                  .join(", ")
              )
            },
            if permission.commands.deny.is_empty() {
              "".to_string()
            } else {
              format!(
                "\n{}: {}",
                "Deny commands".bold(),
                permission
                  .commands
                  .deny
                  .iter()
                  .map(|c| c.red().to_string())
                  .collect::<Vec<_>>()
                  .join(", ")
              )
            },
          ));
        }
      }

      if !permissions.is_empty() {
        println!("{}\n", permissions.join("\n\n"));
      }
    }

    Ok(())
  } else {
    anyhow::bail!("permission file not found, please build your application once first")
  }
}
