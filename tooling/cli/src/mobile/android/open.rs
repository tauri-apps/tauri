// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{ensure_init, env, inject_assets, with_config, MobileTarget};
use crate::{helpers::config::get as get_config, Result};
use tauri_mobile::os;

pub fn command() -> Result<()> {
  with_config(
    Some(Default::default()),
    |_root_conf, config, _metadata, _cli_options| {
      ensure_init(config.project_dir(), MobileTarget::Android)?;
      inject_assets(config, get_config(None)?.lock().unwrap().as_ref().unwrap())?;
      let env = env()?;
      os::open_file_with("Android Studio", config.project_dir(), &env.base).map_err(Into::into)
    },
  )
}
