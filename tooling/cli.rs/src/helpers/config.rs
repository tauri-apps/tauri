// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "linux")]
use heck::KebabCase;
use json_patch::merge;
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;

#[path = "../../config_definition.rs"]
mod config_definition;
pub use config_definition::*;

#[cfg(windows)]
impl From<WixConfig> for tauri_bundler::WixSettings {
  fn from(config: WixConfig) -> tauri_bundler::WixSettings {
    tauri_bundler::WixSettings {
      fragment_paths: config.fragment_paths,
      component_group_refs: config.component_group_refs,
      component_refs: config.component_refs,
      feature_group_refs: config.feature_group_refs,
      feature_refs: config.feature_refs,
      merge_refs: config.merge_refs,
      skip_webview_install: config.skip_webview_install,
    }
  }
}

use std::{
  env::set_var,
  fs::File,
  io::BufReader,
  process::exit,
  sync::{Arc, Mutex},
};

pub type ConfigHandle = Arc<Mutex<Option<Config>>>;

fn config_handle() -> &'static ConfigHandle {
  static CONFING_HANDLE: Lazy<ConfigHandle> = Lazy::new(Default::default);
  &CONFING_HANDLE
}

/// Gets the static parsed config from `tauri.conf.json`.
fn get_internal(merge_config: Option<&str>, reload: bool) -> crate::Result<ConfigHandle> {
  if !reload && config_handle().lock().unwrap().is_some() {
    return Ok(config_handle().clone());
  }

  let path = super::app_paths::tauri_dir().join("tauri.conf.json");
  let file = File::open(path)?;
  let buf = BufReader::new(file);
  let mut config: JsonValue = serde_json::from_reader(buf)?;

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
          .replace("/", " > "),
        error.get_detail().unwrap_or_else(|| error.get_title()),
      );
    }
    exit(1);
  }

  if let Some(merge_config) = merge_config {
    let merge_config: JsonValue = serde_json::from_str(&merge_config)?;
    merge(&mut config, &merge_config);
  }

  #[allow(unused_mut)]
  let mut config: Config = serde_json::from_value(config)?;
  #[cfg(target_os = "linux")]
  if let Some(product_name) = config.package.product_name.as_mut() {
    *product_name = product_name.to_kebab_case();
  }
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
