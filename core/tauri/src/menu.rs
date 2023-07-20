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
  let about_metadata = AboutMetadata {
    name: config.package.product_name.clone(),
    version: config.package.version.clone(),
    copyright: config.tauri.bundle.copyright.clone(),
    authors: config.tauri.bundle.publisher.clone().map(|p| vec![p]),
    ..Default::default()
  };

  Menu::with_items(&[
    #[cfg(target_os = "macos")]
    &Submenu::with_items(
      config.package.binary_name().unwrap_or_default(),
      true,
      &[
        &PredefinedMenuItem::about(None, Some(about_metadata.clone())),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::services(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::hide(None),
        &PredefinedMenuItem::hide_others(None),
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::quit(None),
      ],
    )
    .unwrap(),
    #[cfg(not(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    )))]
    &Submenu::with_items(
      "File",
      true,
      &[
        &PredefinedMenuItem::close_window(None),
        #[cfg(not(target_os = "macos"))]
        &PredefinedMenuItem::quit(None),
      ],
    )
    .unwrap(),
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
    )
    .unwrap(),
    #[cfg(target_os = "macos")]
    &Submenu::with_items("View", true, &[&PredefinedMenuItem::fullscreen(None)]).unwrap(),
    &Submenu::with_items(
      "Window",
      true,
      &[
        &PredefinedMenuItem::minimize(None),
        &PredefinedMenuItem::maximize(None),
        #[cfg(target_os = "macos")]
        &PredefinedMenuItem::separator(),
        &PredefinedMenuItem::close_window(None),
        &PredefinedMenuItem::about(None, Some(about_metadata)),
      ],
    )
    .unwrap(),
  ])
  .unwrap()
}
