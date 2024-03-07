// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt, sync::Mutex};

use crate::{
  app::GlobalTrayIconEventListener,
  image::Image,
  tray::{TrayIcon, TrayIconId},
  AppHandle, Runtime,
};

pub struct TrayManager<R: Runtime> {
  pub(crate) icon: Option<Image<'static>>,
  /// Tray icons
  pub(crate) icons: Mutex<Vec<TrayIcon<R>>>,
  /// Global Tray icon event listeners.
  pub(crate) global_event_listeners: Mutex<Vec<GlobalTrayIconEventListener<AppHandle<R>>>>,
  /// Tray icon event listeners.
  pub(crate) event_listeners: Mutex<HashMap<TrayIconId, GlobalTrayIconEventListener<TrayIcon<R>>>>,
}

impl<R: Runtime> fmt::Debug for TrayManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("TrayManager")
      .field("icon", &self.icon)
      .finish()
  }
}
