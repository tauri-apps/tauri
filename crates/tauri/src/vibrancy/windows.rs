// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use std::ffi::c_void;

use crate::utils::config::WindowEffectsConfig;
use crate::window::{Color, Effect};
use raw_window_handle::HasWindowHandle;
use windows::Win32::Foundation::HWND;

pub fn apply_effects(window: impl HasWindowHandle, effects: WindowEffectsConfig) {
  let WindowEffectsConfig { effects, color, .. } = effects;
  let effect = if let Some(effect) = effects.iter().find(|e| {
    matches!(
      e,
      Effect::Mica
        | Effect::MicaDark
        | Effect::MicaLight
        | Effect::Acrylic
        | Effect::Blur
        | Effect::Tabbed
        | Effect::TabbedDark
        | Effect::TabbedLight
    )
  }) {
    effect
  } else {
    return;
  };

  match effect {
    Effect::Blur => window_vibrancy::apply_blur(window, color.map(Into::into)),
    Effect::Acrylic => window_vibrancy::apply_acrylic(window, color.map(Into::into)),
    Effect::Mica => window_vibrancy::apply_mica(window, None),
    Effect::MicaDark => window_vibrancy::apply_mica(window, Some(true)),
    Effect::MicaLight => window_vibrancy::apply_mica(window, Some(false)),
    Effect::Tabbed => window_vibrancy::apply_tabbed(window, None),
    Effect::TabbedDark => window_vibrancy::apply_tabbed(window, Some(true)),
    Effect::TabbedLight => window_vibrancy::apply_tabbed(window, Some(false)),
    _ => unreachable!(),
  };
}

pub fn clear_effects(window: impl HasWindowHandle) {
  window_vibrancy::clear_blur(&window);
  window_vibrancy::clear_acrylic(&window);
  window_vibrancy::clear_mica(&window);
}
