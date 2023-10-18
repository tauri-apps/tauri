// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{SectionItem, Status};
use crate::helpers::framework;
use std::{
  fs::read_to_string,
  path::{Path, PathBuf},
};
use tauri_utils::platform::Target;

pub fn items(app_dir: Option<&PathBuf>, tauri_dir: Option<&Path>) -> Vec<SectionItem> {
  let mut items = Vec::new();
  if tauri_dir.is_some() {
    if let Ok(config) = crate::helpers::config::get(Target::current(), None) {
      let config_guard = config.lock().unwrap();
      let config = config_guard.as_ref().unwrap();

      let bundle_or_build = if config.tauri.bundle.active {
        "bundle".to_string()
      } else {
        "build".to_string()
      };
      items.push(SectionItem::new(
        move || Some((format!("build-type: {bundle_or_build}"), Status::Neutral)),
        || None,
        false,
      ));

      let csp = config
        .tauri
        .security
        .csp
        .clone()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "unset".to_string());
      items.push(SectionItem::new(
        move || Some((format!("CSP: {csp}"), Status::Neutral)),
        || None,
        false,
      ));

      let dist_dir = config.build.dist_dir.to_string();
      items.push(SectionItem::new(
        move || Some((format!("distDir: {dist_dir}"), Status::Neutral)),
        || None,
        false,
      ));

      let dev_path = config.build.dev_path.to_string();
      items.push(SectionItem::new(
        move || Some((format!("devPath: {dev_path}"), Status::Neutral)),
        || None,
        false,
      ));

      if let Some(app_dir) = app_dir {
        if let Ok(package_json) = read_to_string(app_dir.join("package.json")) {
          let (framework, bundler) = framework::infer_from_package_json(&package_json);
          if let Some(framework) = framework {
            items.push(SectionItem::new(
              move || Some((format!("framework: {framework}"), Status::Neutral)),
              || None,
              false,
            ));
          }
          if let Some(bundler) = bundler {
            items.push(SectionItem::new(
              move || Some((format!("bundler: {bundler}"), Status::Neutral)),
              || None,
              false,
            ));
          }
        }
      }
    }
  }

  items
}
