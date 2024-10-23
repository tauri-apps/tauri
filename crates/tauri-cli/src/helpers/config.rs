// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use json_patch::merge;
use serde_json::Value as JsonValue;

pub use tauri_utils::{config::*, platform::Target};

use std::{
  collections::HashMap,
  env::{current_dir, set_current_dir, set_var, var_os},
  ffi::OsStr,
  process::exit,
  sync::{Arc, Mutex, OnceLock},
};

pub const MERGE_CONFIG_EXTENSION_NAME: &str = "--config";

pub struct ConfigMetadata {
  /// The current target.
  target: Target,
  /// The actual configuration, merged with any extension.
  inner: Config,
  /// The config extensions (platform-specific config files or the config CLI argument).
  /// Maps the extension name to its value.
  extensions: HashMap<String, JsonValue>,
}

impl std::ops::Deref for ConfigMetadata {
  type Target = Config;

  #[inline(always)]
  fn deref(&self) -> &Config {
    &self.inner
  }
}

impl ConfigMetadata {
  /// Checks which config is overwriting the bundle identifier.
  pub fn find_bundle_identifier_overwriter(&self) -> Option<String> {
    for (ext, config) in &self.extensions {
      if let Some(identifier) = config
        .as_object()
        .and_then(|config| config.get("tauri"))
        .and_then(|tauri_config| tauri_config.as_object())
        .and_then(|tauri_config| tauri_config.get("bundle"))
        .and_then(|bundle_config| bundle_config.as_object())
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

pub type ConfigHandle = Arc<Mutex<Option<ConfigMetadata>>>;

pub fn wix_settings(config: WixConfig) -> tauri_bundler::WixSettings {
  tauri_bundler::WixSettings {
    version: config.version,
    upgrade_code: config.upgrade_code,
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
    fips_compliant: var_os("TAURI_BUNDLER_WIX_FIPS_COMPLIANT").map_or(false, |v| v == "true"),
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

fn config_handle() -> &'static ConfigHandle {
  static CONFIG_HANDLE: OnceLock<ConfigHandle> = OnceLock::new();
  CONFIG_HANDLE.get_or_init(Default::default)
}

/// Gets the static parsed config from `tauri.conf.json`.
fn get_internal(
  merge_config: Option<&serde_json::Value>,
  reload: bool,
  target: Target,
) -> crate::Result<ConfigHandle> {
  if !reload && config_handle().lock().unwrap().is_some() {
    return Ok(config_handle().clone());
  }

  let tauri_dir = super::app_paths::tauri_dir();
  let (mut config, config_path) =
    tauri_utils::config::parse::parse_value(target, tauri_dir.join("tauri.conf.json"))?;
  let config_file_name = config_path.file_name().unwrap().to_string_lossy();
  let mut extensions = HashMap::new();

  if let Some((platform_config, config_path)) =
    tauri_utils::config::parse::read_platform(target, tauri_dir.to_path_buf())?
  {
    merge(&mut config, &platform_config);
    extensions.insert(
      config_path.file_name().unwrap().to_str().unwrap().into(),
      platform_config,
    );
  }

  if let Some(merge_config) = merge_config {
    let merge_config_str = serde_json::to_string(&merge_config).unwrap();
    set_var("TAURI_CONFIG", merge_config_str);
    merge(&mut config, merge_config);
    extensions.insert(MERGE_CONFIG_EXTENSION_NAME.into(), merge_config.clone());
  };

  if config_path.extension() == Some(OsStr::new("json"))
    || config_path.extension() == Some(OsStr::new("json5"))
  {
    let schema: JsonValue = serde_json::from_str(include_str!("../../config.schema.json"))?;
    let schema = jsonschema::JSONSchema::compile(&schema).unwrap();
    let result = schema.validate(&config);
    if let Err(errors) = result {
      for error in errors {
        let path = error.instance_path.clone().into_vec().join(" > ");
        if path.is_empty() {
          log::error!("`{}` error: {}", config_file_name, error);
        } else {
          log::error!("`{}` error on `{}`: {}", config_file_name, path, error);
        }
      }
      if !reload {
        exit(1);
      }
    }
  }

  // the `Config` deserializer for `package > version` can resolve the version from a path relative to the config path
  // so we actually need to change the current working directory here
  let current_dir = current_dir()?;
  set_current_dir(config_path.parent().unwrap())?;
  let config: Config = serde_json::from_value(config)?;
  // revert to previous working directory
  set_current_dir(current_dir)?;

  for (plugin, conf) in &config.plugins.0 {
    set_var(
      format!(
        "TAURI_{}_PLUGIN_CONFIG",
        plugin.to_uppercase().replace('-', "_")
      ),
      serde_json::to_string(&conf)?,
    );
  }

  *config_handle().lock().unwrap() = Some(ConfigMetadata {
    target,
    inner: config,
    extensions,
  });

  Ok(config_handle().clone())
}

pub fn get(
  target: Target,
  merge_config: Option<&serde_json::Value>,
) -> crate::Result<ConfigHandle> {
  get_internal(merge_config, false, target)
}

pub fn reload(merge_config: Option<&serde_json::Value>) -> crate::Result<ConfigHandle> {
  let target = config_handle()
    .lock()
    .unwrap()
    .as_ref()
    .map(|conf| conf.target);
  if let Some(target) = target {
    get_internal(merge_config, true, target)
  } else {
    Err(anyhow::anyhow!("config not loaded"))
  }
}

/// merges the loaded config with the given value
pub fn merge_with(merge_config: &serde_json::Value) -> crate::Result<ConfigHandle> {
  let handle = config_handle();
  if let Some(config_metadata) = &mut *handle.lock().unwrap() {
    let merge_config_str = serde_json::to_string(merge_config).unwrap();
    set_var("TAURI_CONFIG", merge_config_str);

    let mut value = serde_json::to_value(config_metadata.inner.clone())?;
    merge(&mut value, merge_config);
    config_metadata.inner = serde_json::from_value(value)?;

    Ok(handle.clone())
  } else {
    Err(anyhow::anyhow!("config not loaded"))
  }
}
