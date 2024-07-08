// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{helpers::app_paths::walk_builder, Result};
use anyhow::Context;

use std::{fs, path::Path};

// (from, to)
const RENAMED_MODULES: &[(&str, &str)] = &[("tauri", "core"), ("window", "webviewWindow")];
const PLUGINIFIED_MODULES: &[&str] = &[
  "cli",
  "clipboard",
  "dialog",
  "fs",
  "globalShortcut",
  "http",
  "notification",
  "os",
  "process",
  "shell",
  "updater",
];
const JS_EXTENSIONS: &[&str] = &["js", "mjs", "jsx", "ts", "mts", "tsx"];

/// Returns a list of paths that could not be migrated
pub fn migrate(app_dir: &Path) -> Result<Vec<String>> {
  let mut new_plugins = Vec::new();

  let tauri_api_import_regex = regex::bytes::Regex::new(r"@tauri-apps/api/(\w+)").unwrap();

  for entry in walk_builder(app_dir).build().flatten() {
    if entry.file_type().map(|t| t.is_file()).unwrap_or_default() {
      let path = entry.path();
      let ext = path.extension().unwrap_or_default();
      if JS_EXTENSIONS.iter().any(|e| e == &ext) {
        let js_contents = fs::read(path)?;

        let new_contents =
          tauri_api_import_regex.replace_all(&js_contents, |cap: &regex::bytes::Captures<'_>| {
            let module = cap.get(1).unwrap().as_bytes();
            let mut module = String::from_utf8_lossy(module).to_string();
            let original = cap.get(0).unwrap().as_bytes();
            let original = String::from_utf8_lossy(original).to_string();

            let new = if let Some((_, renamed_to)) =
              RENAMED_MODULES.iter().find(|(from, _to)| *from == module)
            {
              format!("@tauri-apps/api/{renamed_to}")
            } else if PLUGINIFIED_MODULES.contains(&module.as_str()) {
              match module.as_str() {
                "clipboard" => module = String::from("clipboard-manager"),
                "globalShortcut" => module = String::from("global-shortcut"),
                _ => {}
              }
              let plugin = format!("@tauri-apps/plugin-{module}");
              new_plugins.push(module);
              plugin
            } else {
              return original;
            };

            log::info!("Replacing `{original}` with `{new}` on {}", path.display());
            new
          });

        if new_contents != js_contents {
          fs::write(path, new_contents)
            .with_context(|| format!("Error writing {}", path.display()))?;
        }
      }
    }
  }

  Ok(new_plugins)
}
