// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Menu types and utility functions

use tauri_utils::config::Config;

pub use crate::runtime::menu::*;

// TODO(muda-migration): wrapper types on tauri's side that will implement unsafe sync & send and hide unnecssary APIs
// TODO(muda-migration): figure out js events

/// Creates a menu filled with default menu items and submenus.
pub fn default(config: &Config) -> Menu {
  let mut about_metadata = AboutMetadata::default();
  about_metadata.name = config.package.product_name.clone();
  about_metadata.version = config.package.version.clone();
  about_metadata.copyright = config.tauri.bundle.copyright.clone();
  about_metadata.authors = config.tauri.bundle.publisher.clone().map(|p| vec![p]);

  Menu::with_items(&[
    #[cfg(target_os = "macos")]
    &Submenu::with_items(
      config.package.binary_name().unwrap_or_default(),
      true,
      &[
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
      ],
    ),
    &Submenu::with_items(
      "File",
      true,
      &[
        &PredefinedMenuItem::close_window(None),
        #[cfg(not(target_os = "macos"))]
        &PredefinedMenuItem::quit(None),
      ],
    ),
    &Submenu::with_items(
      "Edit",
      true,
      &[
        &PredefinedMenuItem::undo(None),
        &PredefinedMenuItem::redo(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::cut(None),
        &PredefinedMenuItem::copy(None),
        &PredefinedMenuItem::paste(None),
        &PredefinedMenuItem::select_all(None),
      ],
    ),
    #[cfg(target_os = "macos")]
    &Submenu::with_items("View", true, &[&PredefinedMenuItem::fullscreen(None)]),
    &Submenu::with_items(
      "Window",
      true,
      &[
        &PredefinedMenuItem::minimize(None),
        &PredefinedMenuItem::maximize(None),
        #[cfg(target_os = "macos")]
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::close_window(None),
      ],
    ),
  ])
}
