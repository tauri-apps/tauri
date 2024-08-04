// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ensure_init, env, get_app, get_config, inject_assets, MobileTarget};
use crate::{
  helpers::config::get as get_tauri_config,
  interface::{AppInterface, Interface},
  Result,
};

use cargo_mobile2::os;

pub fn command() -> Result<()> {
  let tauri_config = get_tauri_config(tauri_utils::platform::Target::Android, None)?;

  let (config, _metadata) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    get_config(
      &get_app(tauri_config_, &AppInterface::new(tauri_config_, None)?),
      tauri_config_,
      None,
      &Default::default(),
    )
  };
  ensure_init(&tauri_config, config.project_dir(), MobileTarget::Android)?;
  inject_assets(&config, tauri_config.lock().unwrap().as_ref().unwrap())?;
  let env = env()?;
  os::open_file_with("Android Studio", config.project_dir(), &env.base).map_err(Into::into)
}
