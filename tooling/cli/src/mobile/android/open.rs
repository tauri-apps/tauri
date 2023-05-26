// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ensure_init, env, get_app, get_config, inject_assets, MobileTarget};
use crate::{
  helpers::{app_paths::tauri_dir, config::get as get_tauri_config},
  Result,
};

use anyhow::Context;
use tauri_mobile::os;

use std::env::set_current_dir;

pub fn command() -> Result<()> {
  let tauri_path = tauri_dir();
  set_current_dir(tauri_path).with_context(|| "failed to change current working directory")?;
  let tauri_config = get_tauri_config(None)?;

  let (config, _metadata) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    get_config(&get_app(tauri_config_), tauri_config_, &Default::default())
  };
  ensure_init(config.project_dir(), MobileTarget::Android)?;
  inject_assets(&config, tauri_config.lock().unwrap().as_ref().unwrap())?;
  let env = env()?;
  os::open_file_with("Android Studio", config.project_dir(), &env.base).map_err(Into::into)
}
