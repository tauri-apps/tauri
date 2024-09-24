// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused)]

use tauri_utils::config::WindowEffectsConfig;

use crate::{Runtime, Window};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub fn set_window_effects<R: Runtime>(
  window: &Window<R>,
  effects: Option<WindowEffectsConfig>,
) -> crate::Result<()> {
  if let Some(_effects) = effects {
    #[cfg(windows)]
    windows::apply_effects(window, _effects);
    #[cfg(target_os = "macos")]
    macos::apply_effects(window, _effects);
  } else {
    #[cfg(windows)]
    windows::clear_effects(window);
  }
  Ok(())
}
