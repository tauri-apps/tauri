// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::SectionItem;
use crate::helpers::framework;
use std::{
  fs::read_to_string,
  path::{Path, PathBuf},
};
use tauri_utils::platform::Target;

pub fn items(frontend_dir: Option<&PathBuf>, tauri_dir: Option<&Path>) -> Vec<SectionItem> {
  let mut items = Vec::new();
  if tauri_dir.is_some() {
    if let Ok(config) = crate::helpers::config::get(Target::current(), None) {
      let config_guard = config.lock().unwrap();
      let config = config_guard.as_ref().unwrap();

      let bundle_or_build = if config.bundle.active {
        "bundle"
      } else {
        "build"
      };
      items.push(SectionItem::new().description(format!("build-type: {bundle_or_build}")));

      let csp = config
        .app
        .security
        .csp
        .clone()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "unset".to_string());
      items.push(SectionItem::new().description(format!("CSP: {csp}")));

      if let Some(frontend_dist) = &config.build.frontend_dist {
        items.push(SectionItem::new().description(format!("frontendDist: {frontend_dist}")));
      }

      if let Some(dev_url) = &config.build.dev_url {
        items.push(SectionItem::new().description(format!("devUrl: {dev_url}")));
      }

      if let Some(frontend_dir) = frontend_dir {
        if let Ok(package_json) = read_to_string(frontend_dir.join("package.json")) {
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
