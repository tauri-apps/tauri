// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(deprecated)]

use crate::utils::config::WindowEffectsConfig;
use crate::window::{Effect, EffectState};
use raw_window_handle::HasWindowHandle;
use window_vibrancy::{
  LiquidGlassOptions, NSGlassEffectViewStyle, NSVisualEffectMaterial, NSVisualEffectState,
};

pub fn apply_effects(window: impl HasWindowHandle, effects: WindowEffectsConfig) {
  let WindowEffectsConfig {
    effects,
    radius,
    state,
    color,
    interactive,
  } = effects;

  // window-vibrancy inserts a new subview on every call, so drop the previous effect first
  clear_effects(&window);

  if let Some(effect) = effects
    .iter()
    .find(|e| matches!(e, Effect::LiquidGlassRegular | Effect::LiquidGlassClear))
  {
    let mut options = LiquidGlassOptions::new(match effect {
      Effect::LiquidGlassRegular => NSGlassEffectViewStyle::Regular,
      Effect::LiquidGlassClear => NSGlassEffectViewStyle::Clear,
      _ => unreachable!(),
    })
    .interactive(interactive);

    if let Some(color) = color {
      options = options.tint_color(color.into());
    }
    if let Some(radius) = radius {
      options = options.radius(radius);
    }

    match window_vibrancy::apply_liquid_glass(&window, options) {
      Ok(()) => return,
      // macOS 15 and below: fall back to the Visual Effect material, if any
      Err(window_vibrancy::Error::UnsupportedPlatformVersion(_)) => {}
      Err(e) => {
        log::error!("failed to apply liquid glass effect: {e}");
        return;
      }
    }
  }

  let effect = if let Some(effect) = effects.into_iter().find(|e| {
    matches!(
      e,
      Effect::AppearanceBased
        | Effect::Light
        | Effect::Dark
        | Effect::MediumLight
        | Effect::UltraDark
        | Effect::Titlebar
        | Effect::Selection
        | Effect::Menu
        | Effect::Popover
        | Effect::Sidebar
        | Effect::HeaderView
        | Effect::Sheet
        | Effect::WindowBackground
        | Effect::HudWindow
        | Effect::FullScreenUI
        | Effect::Tooltip
        | Effect::ContentBackground
        | Effect::UnderWindowBackground
        | Effect::UnderPageBackground
    )
  }) {
    effect
  } else {
    return;
  };

  window_vibrancy::apply_vibrancy(
    window,
    match effect {
      Effect::AppearanceBased => NSVisualEffectMaterial::AppearanceBased,
      Effect::Light => NSVisualEffectMaterial::Light,
      Effect::Dark => NSVisualEffectMaterial::Dark,
      Effect::MediumLight => NSVisualEffectMaterial::MediumLight,
      Effect::UltraDark => NSVisualEffectMaterial::UltraDark,
      Effect::Titlebar => NSVisualEffectMaterial::Titlebar,
      Effect::Selection => NSVisualEffectMaterial::Selection,
      Effect::Menu => NSVisualEffectMaterial::Menu,
      Effect::Popover => NSVisualEffectMaterial::Popover,
      Effect::Sidebar => NSVisualEffectMaterial::Sidebar,
      Effect::HeaderView => NSVisualEffectMaterial::HeaderView,
      Effect::Sheet => NSVisualEffectMaterial::Sheet,
      Effect::WindowBackground => NSVisualEffectMaterial::WindowBackground,
      Effect::HudWindow => NSVisualEffectMaterial::HudWindow,
      Effect::FullScreenUI => NSVisualEffectMaterial::FullScreenUI,
      Effect::Tooltip => NSVisualEffectMaterial::Tooltip,
      Effect::ContentBackground => NSVisualEffectMaterial::ContentBackground,
      Effect::UnderWindowBackground => NSVisualEffectMaterial::UnderWindowBackground,
      Effect::UnderPageBackground => NSVisualEffectMaterial::UnderPageBackground,
      _ => unreachable!(),
    },
    state.map(|s| match s {
      EffectState::FollowsWindowActiveState => NSVisualEffectState::FollowsWindowActiveState,
      EffectState::Active => NSVisualEffectState::Active,
      EffectState::Inactive => NSVisualEffectState::Inactive,
    }),
    radius,
  );
}

pub fn clear_effects(window: impl HasWindowHandle) {
  let _ = window_vibrancy::clear_vibrancy(&window);
  let _ = window_vibrancy::clear_liquid_glass(&window);
}
