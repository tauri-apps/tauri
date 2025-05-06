// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  fmt,
  sync::{Arc, Mutex},
};

use crate::{
  app::GlobalTrayIconEventListener,
  image::Image,
  tray::{TrayIcon, TrayIconEvent, TrayIconId},
  AppHandle, Manager, Resource, ResourceId, Runtime,
};

pub struct TrayManager<R: Runtime> {
  pub(crate) icon: Option<Image<'static>>,
  /// Tray icons
  pub(crate) icons: Mutex<Vec<(TrayIconId, ResourceId)>>,
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

  pub fn tray_by_id<'a, I>(&self, app: &AppHandle<R>, id: &'a I) -> Option<TrayIcon<R>>
  where
    I: ?Sized,
    TrayIconId: PartialEq<&'a I>,
  {
    let icons = self.icons.lock().unwrap();
    icons.iter().find_map(|(tray_icon_id, rid)| {
      if tray_icon_id == &id {
        let icon = app.resources_table().get::<TrayIcon<R>>(*rid).ok()?;
        Some(Arc::unwrap_or_clone(icon))
      } else {
        None
      }
    })
  }

  pub fn tray_resource_by_id<'a, I>(&self, id: &'a I) -> Option<ResourceId>
  where
    I: ?Sized,
    TrayIconId: PartialEq<&'a I>,
  {
    let icons = self.icons.lock().unwrap();
    icons.iter().find_map(|(tray_icon_id, rid)| {
      if tray_icon_id == &id {
        Some(*rid)
      } else {
        None
      }
    })
  }

  pub fn remove_tray_by_id<'a, I>(&self, app: &AppHandle<R>, id: &'a I) -> Option<TrayIcon<R>>
  where
    I: ?Sized,
    TrayIconId: PartialEq<&'a I>,
  {
    let rid = self.tray_resource_by_id(id)?;
    let icon = app.resources_table().take::<TrayIcon<R>>(rid).ok()?;
    let icon_to_return = icon.clone();
    icon.close();
    Some(Arc::unwrap_or_clone(icon_to_return))
  }
}
