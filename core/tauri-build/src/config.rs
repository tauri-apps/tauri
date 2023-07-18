use serde::de::DeserializeOwned;

use std::{env::var, io::Cursor};

pub fn plugin_config<T: DeserializeOwned>(name: &str) -> Option<T> {
  if let Ok(config_str) = var(format!("TAURI_{name}_PLUGIN_CONFIG")) {
    serde_json::from_reader(Cursor::new(config_str))
      .map(Some)
      .expect("failed to parse configuration")
  } else {
    None
  }
}
