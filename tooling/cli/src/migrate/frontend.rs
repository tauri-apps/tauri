// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{app_paths::walk_builder, cargo, npm::PackageManager},
  Result,
};

use std::{
  fs::{read_to_string, write},
  path::Path,
};

const CORE_API_MODULES: &[&str] = &["dpi", "event", "path", "core", "window", "mocks"];
const JS_EXTENSIONS: &[&str] = &["js", "jsx", "ts", "tsx", "mjs"];

pub fn migrate(app_dir: &Path, tauri_dir: &Path) -> Result<()> {
  let mut new_npm_packages = Vec::new();
  let mut new_cargo_packages = Vec::new();

  let pm = PackageManager::from_project(app_dir)
    .into_iter()
    .next()
    .unwrap_or(PackageManager::Npm);

  let tauri_api_import_regex = regex::Regex::new(r"@tauri-apps/api/(\w+)").unwrap();

  for entry in walk_builder(app_dir).build().flatten() {
    if entry.file_type().map(|t| t.is_file()).unwrap_or_default() {
      let path = entry.path();
      let ext = path.extension().unwrap_or_default();
      if JS_EXTENSIONS.iter().any(|e| e == &ext) {
        let js_contents = read_to_string(path)?;

        let new_contents =
          tauri_api_import_regex.replace_all(&js_contents, |cap: &regex::Captures<'_>| {
            let module = cap.get(1).unwrap().as_str();
            let original = cap.get(0).unwrap().as_str();

            if module == "tauri" {
              let new = "@tauri-apps/api/core".to_string();
              log::info!("Replacing `{original}` with `{new}` on {}", path.display());
              new
            } else if module == "window" {
              let new = "@tauri-apps/api/webviewWindow".to_string();
              log::info!("Replacing `{original}` with `{new}` on {}", path.display());
              new
            } else if module == "globalShortcut" {
              let new = "@tauri-apps/plugin-global-shortcut".to_string();
              log::info!("Replacing `{original}` with `{new}` on {}", path.display());
              new
            } else if CORE_API_MODULES.contains(&module) {
              original.to_string()
            } else {
              let plugin = format!("@tauri-apps/plugin-{module}");
              log::info!(
                "Replacing `{original}` with `{plugin}` on {}",
                path.display()
              );

              new_npm_packages.push(plugin.clone());
              new_cargo_packages.push(format!(
                "tauri-plugin-{}",
                if module == "clipboard" {
                  "clipboard-manager"
                } else {
                  module
                }
              ));

              plugin
            }
          });

        if new_contents != js_contents {
          write(path, new_contents.as_bytes())?;
        }
      }
    }
  }

  if !new_npm_packages.is_empty() {
    pm.install(&new_npm_packages)?;
  }

  if !new_cargo_packages.is_empty() {
    cargo::install(&new_cargo_packages, Some(tauri_dir))?;
  }

  Ok(())
}
