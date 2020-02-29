use serde::Deserialize;
use super::category::AppCategory;
use std::path::PathBuf;

use std::fs;

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "bundle", rename_all = "camelCase")]
pub struct BundleConfig {
  pub name: Option<String>,
  pub identifier: Option<String>,
  pub icon: Option<Vec<String>>,
  pub version: Option<String>,
  pub resources: Option<Vec<String>>,
  pub copyright: Option<String>,
  pub category: Option<AppCategory>,
  pub short_description: Option<String>,
  pub long_description: Option<String>,
  pub script: Option<PathBuf>,
  // OS-specific settings:
  pub deb_depends: Option<Vec<String>>,
  pub osx_frameworks: Option<Vec<String>>,
  pub osx_minimum_system_version: Option<String>,
  pub external_bin: Option<Vec<String>>,
  pub exception_domain: Option<String>,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  #[serde(default = "default_bundle_config")]
  pub bundle: BundleConfig,
}

fn default_bundle_config() -> BundleConfig {
  BundleConfig {
    name: None,
    identifier: None,
    icon: None,
    version: None,
    resources: None,
    copyright: None,
    category: None,
    short_description: None,
    long_description: None,
    script: None,
    deb_depends: None,
    osx_frameworks: None,
    osx_minimum_system_version: None,
    external_bin: None,
    exception_domain: None,
  }
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default = "default_tauri_config")]
  pub tauri: TauriConfig,
}

fn default_tauri_config() -> TauriConfig {
  TauriConfig {
    bundle: default_bundle_config(),
  }
}

pub fn get() -> crate::Result<Config> {
  match std::env::var_os("TAURI_CONFIG") {
    Some(config) => {
      let json = &config.into_string().expect("failed to read TAURI_CONFIG");
      Ok(serde_json::from_str(json)?)
    },
    None => match std::env::var_os("TAURI_DIR") {
      Some(tauri_dir) => {
        let tauri_dir_str = tauri_dir.into_string().expect("failed to read TAURI_DIR");
        let json = &fs::read_to_string(format!("{}{}", tauri_dir_str, "/tauri.conf.json"))?;
        Ok(serde_json::from_str(json)?)
      },
      None => Err(crate::Error::from("Couldn't get tauri config; please specify the TAURI_CONFIG or TAURI_DIR environment variables"))
    }
  }
}