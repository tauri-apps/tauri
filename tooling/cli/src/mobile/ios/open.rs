// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ensure_init, env, get_app, get_config, MobileTarget};
use crate::{helpers::config::get as get_tauri_config, Result};

use tauri_mobile::os;

pub fn command() -> Result<()> {
  let tauri_config = get_tauri_config(None)?;

  let (config, _metadata) = {
    let tauri_config_guard = tauri_config.lock().unwrap();
    let tauri_config_ = tauri_config_guard.as_ref().unwrap();
    get_config(&get_app(tauri_config_), tauri_config_, &Default::default())
  };

  ensure_init(config.project_dir(), MobileTarget::Ios)?;
  let env = env()?;
  os::open_file_with("Xcode", config.project_dir(), &env).map_err(Into::into)
}
