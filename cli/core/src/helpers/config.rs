use json_patch::merge;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use std::{
  collections::HashMap,
  fs::File,
  io::BufReader,
  sync::{Arc, Mutex},
};

pub type ConfigHandle = Arc<Mutex<Option<Config>>>;

fn config_handle() -> &'static ConfigHandle {
  static CONFING_HANDLE: Lazy<ConfigHandle> = Lazy::new(Default::default);
  &CONFING_HANDLE
}

/// The bundler configuration object.
#[derive(PartialEq, Clone, Deserialize, Serialize, Debug)]
#[serde(tag = "bundle", rename_all = "camelCase")]
pub struct BundleConfig {
  #[serde(default)]
  pub active: bool,
  /// The bundle identifier.
  pub identifier: String,
}

fn default_bundle() -> BundleConfig {
  BundleConfig {
    active: false,
    identifier: String::from(""),
  }
}

/// The Tauri configuration object.
#[derive(PartialEq, Clone, Deserialize, Serialize, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  /// The CLI configuration.
  #[serde(default)]
  pub cli: Option<JsonValue>,
  /// The bundler configuration.
  #[serde(default = "default_bundle")]
  pub bundle: BundleConfig,
  #[serde(default)]
  pub allowlist: HashMap<String, bool>,
}

/// The Build configuration object.
#[derive(PartialEq, Clone, Deserialize, Serialize, Debug)]
#[serde(tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  /// the devPath config.
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
  #[serde(default = "default_dist_dir")]
  pub dist_dir: String,
  pub before_dev_command: Option<String>,
  pub before_build_command: Option<String>,
  #[serde(default)]
  pub with_global_tauri: bool,
}

fn default_dev_path() -> String {
  "".to_string()
}

fn default_dist_dir() -> String {
  "../dist".to_string()
}

type JsonObject = HashMap<String, JsonValue>;

/// The tauri.conf.json mapper.
#[derive(PartialEq, Clone, Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  /// The Tauri configuration.
  #[serde(default = "default_tauri")]
  pub tauri: TauriConfig,
  /// The build configuration.
  #[serde(default = "default_build")]
  pub build: BuildConfig,
  /// The plugins config.
  #[serde(default)]
  pub plugins: HashMap<String, JsonObject>,
}

fn default_tauri() -> TauriConfig {
  TauriConfig {
    cli: None,
    bundle: default_bundle(),
    allowlist: Default::default(),
  }
}

fn default_build() -> BuildConfig {
  BuildConfig {
    dev_path: default_dev_path(),
    dist_dir: default_dist_dir(),
    before_dev_command: None,
    before_build_command: None,
    with_global_tauri: false,
  }
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

  if let Some(merge_config) = merge_config {
    let merge_config: JsonValue = serde_json::from_str(&merge_config)?;
    merge(&mut config, &merge_config);
  }

  let config = serde_json::from_value(config)?;
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
