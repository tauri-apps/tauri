use serde::Deserialize;

use std::fs;

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "bundle", rename_all = "camelCase")]
pub struct BundleConfig {
  pub resources: Option<Vec<String>>,
  pub external_bin: Option<Vec<String>>,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  pub bundle: Option<BundleConfig>,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  pub tauri: Option<TauriConfig>,
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