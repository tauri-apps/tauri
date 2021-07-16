// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use anyhow::Context;
#[cfg(target_os = "linux")]
use heck::KebabCase;
use json_patch::merge;
use once_cell::sync::Lazy;
use serde_json::Value as JsonValue;

#[path = "../../config_definition.rs"]
mod config_definition;
pub use config_definition::*;

impl From<WixConfig> for tauri_bundler::WixSettings {
  fn from(config: WixConfig) -> tauri_bundler::WixSettings {
    tauri_bundler::WixSettings {
      language: config.language,
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
  let mut config: JsonValue =
    serde_json::from_reader(buf).with_context(|| "failed to parse `tauri.conf.json`")?;

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
    let merge_config: JsonValue =
      serde_json::from_str(merge_config).with_context(|| "failed to parse config to merge")?;
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

pub fn all_allowlist_features() -> Vec<&'static str> {
  AllowlistConfig {
    all: true,
    fs: FsAllowlistConfig {
      all: true,
      read_text_file: true,
      read_binary_file: true,
      write_file: true,
      write_binary_file: true,
      read_dir: true,
      copy_file: true,
      create_dir: true,
      remove_dir: true,
      remove_file: true,
      rename_file: true,
      path: true,
    },
    window: WindowAllowlistConfig {
      all: true,
      create: true,
    },
    shell: ShellAllowlistConfig {
      all: true,
      execute: true,
      open: true,
    },
    dialog: DialogAllowlistConfig {
      all: true,
      open: true,
      save: true,
    },
    http: HttpAllowlistConfig {
      all: true,
      request: true,
    },
    notification: NotificationAllowlistConfig { all: true },
    global_shortcut: GlobalShortcutAllowlistConfig { all: true },
  }
  .to_features()
}
