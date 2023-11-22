// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::SectionItem;
use crate::helpers::framework;
use std::{fs::read_to_string, path::PathBuf};

pub fn items(app_dir: Option<&PathBuf>, tauri_dir: Option<PathBuf>) -> Vec<SectionItem> {
  let mut items = Vec::new();
  if tauri_dir.is_some() {
    if let Ok(config) = crate::helpers::config::get(None) {
      let config_guard = config.lock().unwrap();
      let config = config_guard.as_ref().unwrap();

      let bundle_or_build = if config.tauri.bundle.active {
        "bundle"
      } else {
        "build"
      };
      items.push(SectionItem::new().description(format!("build-type: {bundle_or_build}")));

      let csp = config
        .tauri
        .security
        .csp
        .clone()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "unset".to_string());
      items.push(SectionItem::new().description(format!("CSP: {csp}")));

      let dist_dir = &config.build.dist_dir;
      items.push(SectionItem::new().description(format!("distDir: {dist_dir}")));

      let dev_path = &config.build.dev_path;
      items.push(SectionItem::new().description(format!("devPath: {dev_path}")));

      if let Some(app_dir) = app_dir {
        if let Ok(package_json) = read_to_string(app_dir.join("package.json")) {
          let (framework, bundler) = framework::infer_from_package_json(&package_json);

          if let Some(framework) = framework {
            items.push(SectionItem::new().description(format!("framework: {framework}")));
          }

          if let Some(bundler) = bundler {
            items.push(SectionItem::new().description(format!("bundler: {bundler}")));
          }
        }
      }
    }
  }

  items
}
