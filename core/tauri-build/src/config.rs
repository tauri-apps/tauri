// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::de::DeserializeOwned;

use std::{env::var, io::Cursor};

pub fn plugin_config<T: DeserializeOwned>(name: &str) -> Option<T> {
  let config_env_var_name = format!(
    "TAURI_{}_PLUGIN_CONFIG",
    name.to_uppercase().replace('-', "_")
  );
  if let Ok(config_str) = var(&config_env_var_name) {
    println!("cargo:rerun-if-env-changed={config_env_var_name}");
    serde_json::from_reader(Cursor::new(config_str))
      .map(Some)
      .expect("failed to parse configuration")
  } else {
    None
  }
}
