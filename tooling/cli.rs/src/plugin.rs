// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{resolve_tauri_path, template, Logger},
  VersionMetadata,
};
use anyhow::Context;
use handlebars::{to_json, Handlebars};
use heck::{KebabCase, SnakeCase};
use include_dir::{include_dir, Dir};

use std::{collections::BTreeMap, env::current_dir, fs::remove_dir_all, path::PathBuf};

const BACKEND_PLUGIN_DIR: Dir = include_dir!("templates/plugin/backend");
const API_PLUGIN_DIR: Dir = include_dir!("templates/plugin/with-api");

pub struct Plugin {
  plugin_name: String,
  api: bool,
  directory: PathBuf,
  tauri_path: Option<PathBuf>,
}

impl Default for Plugin {
  fn default() -> Self {
    Self {
      plugin_name: "".into(),
      api: false,
      directory: current_dir().expect("failed to read cwd"),
      tauri_path: None,
    }
  }
}

impl Plugin {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn plugin_name(mut self, plugin_name: String) -> Self {
    self.plugin_name = plugin_name;
    self
  }

  pub fn api(mut self) -> Self {
    self.api = true;
    self
  }

  pub fn directory(mut self, directory: impl Into<PathBuf>) -> Self {
    self.directory = directory.into();
    self
  }

  pub fn tauri_path(mut self, tauri_path: impl Into<PathBuf>) -> Self {
    self.tauri_path = Some(tauri_path.into());
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let logger = Logger::new("tauri:init:plugin");
    let template_target_path = self.directory.join(&format!(
      "tauri-plugin-{}",
      self.plugin_name.to_kebab_case()
    ));
    let metadata = serde_json::from_str::<VersionMetadata>(include_str!("../metadata.json"))?;
    if template_target_path.exists() {
      logger.warn(format!(
        "Plugin dir ({:?}) not empty.",
        template_target_path
      ));
    } else {
      let (tauri_dep, tauri_build_dep) = if let Some(tauri_path) = self.tauri_path {
        (
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri")
          ),
          format!(
            "{{  path = {:?} }}",
            resolve_tauri_path(&tauri_path, "core/tauri-build")
          ),
        )
      } else {
        (
          format!(
            r#"{{ version = "{}", features = [ "api-all" ] }}"#,
            metadata.tauri
          ),
          format!(
            r#"{{ version = "{}", features = [ "api-all" ] }}"#,
            metadata.tauri_build
          ),
        )
      };

      let _ = remove_dir_all(&template_target_path);
      let handlebars = Handlebars::new();

      let mut data = BTreeMap::new();
      data.insert("plugin_name_original", to_json(&self.plugin_name));
      data.insert("plugin_name", to_json(self.plugin_name.to_kebab_case()));
      data.insert(
        "plugin_name_snake_case",
        to_json(self.plugin_name.to_snake_case()),
      );
      data.insert("tauri_dep", to_json(tauri_dep));
      data.insert("tauri_build_dep", to_json(tauri_build_dep));

      template::render(
        &handlebars,
        &data,
        if self.api {
          &API_PLUGIN_DIR
        } else {
          &BACKEND_PLUGIN_DIR
        },
        &template_target_path,
      )
      .with_context(|| "failed to render Tauri template")?;
    }
    Ok(())
  }
}
