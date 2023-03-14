// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
    {
      let hwnd = window.hwnd()?;
      windows::apply_effects(hwnd, _effects);
    }
    #[cfg(target_os = "macos")]
    {
      let ns_window = window.ns_window()?;
      macos::apply_effects(ns_window as _, _effects);
    }
  } else {
    #[cfg(windows)]
    {
      let hwnd = window.hwnd()?;
      windows::clear_effects(hwnd);
    }
  }
  Ok(())
}
