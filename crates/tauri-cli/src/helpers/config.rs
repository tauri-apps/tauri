// Copyright 2019-2025 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use itertools::Itertools;
use json_patch::merge;
use serde_json::Value as JsonValue;

use tauri_utils::acl::REMOVE_UNUSED_COMMANDS_ENV_VAR;
pub use tauri_utils::{config::*, platform::Target};

use std::{
  collections::HashMap,
  env::{current_dir, set_current_dir, set_var},
  ffi::{OsStr, OsString},
  process::exit,
  sync::{Mutex, OnceLock},
};

use crate::error::Context;

pub const MERGE_CONFIG_EXTENSION_NAME: &str = "--config";

pub struct ConfigMetadata {
  /// The current target.
  target: Target,

  original_identifier: Option<String>,
  /// The actual configuration, merged with any extension.
  inner: Config,
  /// The config extensions (platform-specific config files or the config CLI argument).
  /// Maps the extension name to its value.
  extensions: HashMap<OsString, JsonValue>,
}

impl std::ops::Deref for ConfigMetadata {
  type Target = Config;

  #[inline(always)]
  fn deref(&self) -> &Config {
    &self.inner
  }
}

impl ConfigMetadata {
  /// The original bundle identifier from the config file.
  /// This does not take any extensions into account.
  pub fn original_identifier(&self) -> Option<&str> {
    self.original_identifier.as_deref()
  }

  /// Checks which config is overwriting the bundle identifier.
  pub fn find_bundle_identifier_overwriter(&self) -> Option<OsString> {
    for (ext, config) in &self.extensions {
      if let Some(identifier) = config
        .as_object()
        .and_then(|bundle_config| bundle_config.get("identifier"))
        .and_then(|id| id.as_str())
      {
        if identifier == self.inner.identifier {
          return Some(ext.clone());
        }
      }
    }
    None
  }
}

pub type ConfigHandle = &'static Mutex<Option<ConfigMetadata>>;

pub fn wix_settings(config: WixConfig) -> tauri_bundler::WixSettings {
  tauri_bundler::WixSettings {
    version: config.version,
    upgrade_code: config.upgrade_code,
    fips_compliant: std::env::var("TAURI_BUNDLER_WIX_FIPS_COMPLIANT")
      .ok()
      .map(|v| v == "true")
      .unwrap_or(config.fips_compliant),
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
    enable_elevated_update_task: config.enable_elevated_update_task,
    banner_path: config.banner_path,
    dialog_image_path: config.dialog_image_path,
  }
}

pub fn nsis_settings(config: NsisConfig) -> tauri_bundler::NsisSettings {
  tauri_bundler::NsisSettings {
    template: config.template,
    header_image: config.header_image,
    sidebar_image: config.sidebar_image,
    installer_icon: config.installer_icon,
    install_mode: config.install_mode,
    languages: config.languages,
    custom_language_files: config.custom_language_files,
    display_language_selector: config.display_language_selector,
    compression: config.compression,
    start_menu_folder: config.start_menu_folder,
    installer_hooks: config.installer_hooks,
    minimum_webview2_version: config.minimum_webview2_version,
  }
}

pub fn custom_sign_settings(
  config: CustomSignCommandConfig,
) -> tauri_bundler::CustomSignCommandSettings {
  match config {
    CustomSignCommandConfig::Command(command) => {
      let mut tokens = command.split(' ');
      tauri_bundler::CustomSignCommandSettings {
        cmd: tokens.next().unwrap().to_string(), // split always has at least one element
        args: tokens.map(String::from).collect(),
      }
    }
    CustomSignCommandConfig::CommandWithOptions { cmd, args } => {
      tauri_bundler::CustomSignCommandSettings { cmd, args }
    }
  }
}

fn config_handle() -> ConfigHandle {
  static CONFIG_HANDLE: Mutex<Option<ConfigMetadata>> = Mutex::new(None);
  &CONFIG_HANDLE
}

fn config_schema_validator() -> &'static jsonschema::Validator {
  // TODO: Switch to `LazyLock` when we bump MSRV to above 1.80
  static CONFIG_SCHEMA_VALIDATOR: OnceLock<jsonschema::Validator> = OnceLock::new();
  CONFIG_SCHEMA_VALIDATOR.get_or_init(|| {
    let schema: JsonValue = serde_json::from_str(include_str!("../../config.schema.json"))
      .expect("Failed to parse config schema bundled in the tauri-cli");
    jsonschema::validator_for(&schema).expect("Config schema bundled in the tauri-cli is invalid")
  })
}

/// Gets the static parsed config from `tauri.conf.json`.
fn get_internal(
  merge_configs: &[&serde_json::Value],
  reload: bool,
  target: Target,
) -> crate::Result<ConfigHandle> {
  if !reload && config_handle().lock().unwrap().is_some() {
    return Ok(config_handle());
  }

  let tauri_dir = super::app_paths::tauri_dir();
  let (mut config, config_path) =
    tauri_utils::config::parse::parse_value(target, tauri_dir.join("tauri.conf.json"))
      .context("failed to parse config")?;
  let config_file_name = config_path.file_name().unwrap();
  let mut extensions = HashMap::new();

  let original_identifier = config
    .as_object()
    .and_then(|config| config.get("identifier"))
    .and_then(|id| id.as_str())
    .map(ToString::to_string);

  if let Some((platform_config, config_path)) =
    tauri_utils::config::parse::read_platform(target, tauri_dir)
      .context("failed to parse platform config")?
  {
    merge(&mut config, &platform_config);
    extensions.insert(config_path.file_name().unwrap().into(), platform_config);
  }

  if !merge_configs.is_empty() {
    let mut merge_config = serde_json::Value::Object(Default::default());
    for conf in merge_configs {
      merge_patches(&mut merge_config, conf);
    }

    let merge_config_str = serde_json::to_string(&merge_config).unwrap();
    set_var("TAURI_CONFIG", merge_config_str);
    merge(&mut config, &merge_config);
    extensions.insert(MERGE_CONFIG_EXTENSION_NAME.into(), merge_config);
  }

  if config_path.extension() == Some(OsStr::new("json"))
    || config_path.extension() == Some(OsStr::new("json5"))
  {
    let mut errors = config_schema_validator().iter_errors(&config).peekable();
    if errors.peek().is_some() {
      for error in errors {
        let path = error.instance_path.into_iter().join(" > ");
        if path.is_empty() {
          log::error!("`{config_file_name:?}` error: {error}");
        } else {
          log::error!("`{config_file_name:?}` error on `{path}`: {error}");
        }
      }
      if !reload {
        exit(1);
      }
    }
  }

  // the `Config` deserializer for `package > version` can resolve the version from a path relative to the config path
  // so we actually need to change the current working directory here
  let current_dir = current_dir().context("failed to resolve current directory")?;
  set_current_dir(config_path.parent().unwrap()).context("failed to set current directory")?;
  let config: Config = serde_json::from_value(config).context("failed to parse config")?;
  // revert to previous working directory
  set_current_dir(current_dir).context("failed to set current directory")?;

  for (plugin, conf) in &config.plugins.0 {
    set_var(
      format!(
        "TAURI_{}_PLUGIN_CONFIG",
        plugin.to_uppercase().replace('-', "_")
      ),
      serde_json::to_string(&conf).context("failed to serialize config")?,
    );
  }

  if config.build.remove_unused_commands {
    std::env::set_var(REMOVE_UNUSED_COMMANDS_ENV_VAR, tauri_dir);
  }

  *config_handle().lock().unwrap() = Some(ConfigMetadata {
    target,
    original_identifier,
    inner: config,
    extensions,
  });

  Ok(config_handle())
}

pub fn get(target: Target, merge_configs: &[&serde_json::Value]) -> crate::Result<ConfigHandle> {
  get_internal(merge_configs, false, target)
}

pub fn reload(merge_configs: &[&serde_json::Value]) -> crate::Result<ConfigHandle> {
  let target = config_handle()
    .lock()
    .unwrap()
    .as_ref()
    .map(|conf| conf.target);
  if let Some(target) = target {
    get_internal(merge_configs, true, target)
  } else {
    crate::error::bail!("config not loaded");
  }
}

/// merges the loaded config with the given value
pub fn merge_with(merge_configs: &[&serde_json::Value]) -> crate::Result<ConfigHandle> {
  let handle = config_handle();

  if merge_configs.is_empty() {
    return Ok(handle);
  }

  if let Some(config_metadata) = &mut *handle.lock().unwrap() {
    let mut merge_config = serde_json::Value::Object(Default::default());
    for conf in merge_configs {
      merge_patches(&mut merge_config, conf);
    }

    let merge_config_str = serde_json::to_string(&merge_config).unwrap();
    set_var("TAURI_CONFIG", merge_config_str);

    let mut value =
      serde_json::to_value(config_metadata.inner.clone()).context("failed to serialize config")?;
    merge(&mut value, &merge_config);
    config_metadata.inner = serde_json::from_value(value).context("failed to parse config")?;

    Ok(handle)
  } else {
    crate::error::bail!("config not loaded");
  }
}

/// Same as [`json_patch::merge`] but doesn't delete the key when the patch's value is `null`
fn merge_patches(doc: &mut serde_json::Value, patch: &serde_json::Value) {
  use serde_json::{Map, Value};

  if !patch.is_object() {
    *doc = patch.clone();
    return;
  }

  if !doc.is_object() {
    *doc = Value::Object(Map::new());
  }
  let map = doc.as_object_mut().unwrap();
  for (key, value) in patch.as_object().unwrap() {
    merge_patches(map.entry(key.as_str()).or_insert(Value::Null), value);
  }
}

#[cfg(test)]
mod tests {
  #[test]
  fn merge_patches() {
    let mut json = serde_json::Value::Object(Default::default());

    super::merge_patches(
      &mut json,
      &serde_json::json!({
        "app": {
          "withGlobalTauri": true,
          "windows": []
        },
        "plugins": {
          "test": "tauri"
        },
        "build": {
          "devUrl": "http://localhost:8080"
        }
      }),
    );

    super::merge_patches(
      &mut json,
      &serde_json::json!({
        "app": { "withGlobalTauri": null }
      }),
    );

    super::merge_patches(
      &mut json,
      &serde_json::json!({
        "app": { "windows": null }
      }),
    );

    super::merge_patches(
      &mut json,
      &serde_json::json!({
        "plugins": { "updater": {
          "endpoints": ["https://tauri.app"]
        } }
      }),
    );

    assert_eq!(
      json,
      serde_json::json!({
        "app": {
          "withGlobalTauri": null,
          "windows": null
        },
        "plugins": {
          "test": "tauri",
          "updater": {
            "endpoints": ["https://tauri.app"]
          }
        },
        "build": {
          "devUrl": "http://localhost:8080"
        }
      })
    )
  }
}
