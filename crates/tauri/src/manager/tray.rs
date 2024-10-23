// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{collections::HashMap, fmt, sync::Mutex};

use crate::{
  app::GlobalTrayIconEventListener,
  image::Image,
  tray::{TrayIcon, TrayIconEvent, TrayIconId},
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

impl<R: Runtime> TrayManager<R> {
  pub fn on_tray_icon_event<F: Fn(&AppHandle<R>, TrayIconEvent) + Send + Sync + 'static>(
    &self,
    handler: F,
  ) {
    self
      .global_event_listeners
      .lock()
      .unwrap()
      .push(Box::new(handler));
  }

  pub fn tray_by_id<'a, I>(&self, id: &'a I) -> Option<TrayIcon<R>>
  where
    I: ?Sized,
    TrayIconId: PartialEq<&'a I>,
  {
    self
      .icons
      .lock()
      .unwrap()
      .iter()
      .find(|t| t.id() == &id)
      .cloned()
  }

  pub fn remove_tray_by_id<'a, I>(&self, id: &'a I) -> Option<TrayIcon<R>>
  where
    I: ?Sized,
    TrayIconId: PartialEq<&'a I>,
  {
    let mut icons = self.icons.lock().unwrap();
    let idx = icons.iter().position(|t| t.id() == &id);
    if let Some(idx) = idx {
      return Some(icons.swap_remove(idx));
    }
    None
  }
}
