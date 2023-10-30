// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt, sync::Mutex};

use crate::{
  app::GlobalTrayIconEventListener,
  tray::{TrayIcon, TrayIconId},
  AppHandle, Icon, Runtime,
};

pub struct TrayManager<R: Runtime> {
  pub(crate) icon: Option<Icon>,
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
