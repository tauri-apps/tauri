// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;
use json_patch::merge;
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;

pub use tauri_utils::config::*;

use std::{
  env::set_var,
  process::exit,
  sync::{Arc, Mutex},
};

pub type ConfigHandle = Arc<Mutex<Option<Config>>>;

pub fn wix_settings(config: WixConfig) -> tauri_bundler::WixSettings {
  tauri_bundler::WixSettings {
    language: tauri_bundler::WixLanguage(match config.language {
      WixLanguage::One(lang) => vec![(lang, Default::default())],
      WixLanguage::List(languages) => languages
        .into_iter()
        .map(|lang| (lang, Default::default()))
        .collect(),
      WixLanguage::Localized(languages) => languages
        .into_iter()
        .map(|(lang, config)| {
          (
            lang,
            tauri_bundler::WixLanguageConfig {
              locale_path: config.locale_path.map(Into::into),
            },
          )
        })
        .collect(),
    }),
    template: config.template,
    fragment_paths: config.fragment_paths,
    component_group_refs: config.component_group_refs,
    component_refs: config.component_refs,
    feature_group_refs: config.feature_group_refs,
    feature_refs: config.feature_refs,
    merge_refs: config.merge_refs,
    skip_webview_install: config.skip_webview_install,
    license: config.license,
    enable_elevated_update_task: config.enable_elevated_update_task,
    banner_path: config.banner_path,
    dialog_image_path: config.dialog_image_path,
  }
}

fn config_handle() -> &'static ConfigHandle {
  static CONFING_HANDLE: Lazy<ConfigHandle> = Lazy::new(Default::default);
  &CONFING_HANDLE
}

/// Gets the static parsed config from `tauri.conf.json`.
fn get_internal(merge_config: Option<&str>, reload: bool) -> crate::Result<ConfigHandle> {
  if !reload && config_handle().lock().unwrap().is_some() {
    return Ok(config_handle().clone());
  }

  let mut config = tauri_utils::config::parse::read_from(super::app_paths::tauri_dir())?;

  if let Some(merge_config) = merge_config {
    let merge_config: JsonValue =
      serde_json::from_str(merge_config).with_context(|| "failed to parse config to merge")?;
    merge(&mut config, &merge_config);
  }

  let schema: JsonValue = serde_json::from_str(include_str!("../../schema.json"))?;
  let mut scope = valico::json_schema::Scope::new();
  let schema = scope.compile_and_return(schema, false).unwrap();
  let state = schema.validate(&config);
  if !state.errors.is_empty() {
    for error in state.errors {
      eprintln!(
        "`tauri.conf.json` error on `{}`: {}",
        error
          .get_path()
          .chars()
          .skip(1)
          .collect::<String>()
          .replace('/', " > "),
        error.get_detail().unwrap_or_else(|| error.get_title()),
      );
    }
    exit(1);
  }

  let config: Config = serde_json::from_value(config)?;
  set_var("TAURI_CONFIG", serde_json::to_string(&config)?);
  *config_handle().lock().unwrap() = Some(config);

  Ok(config_handle().clone())
}

pub fn get(merge_config: Option<&str>) -> crate::Result<ConfigHandle> {
  get_internal(merge_config, false)
}

pub fn reload(merge_config: Option<&str>) -> crate::Result<()> {
  get_internal(merge_config, true)?;
  Ok(())
}
