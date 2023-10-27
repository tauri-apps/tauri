use std::{
  collections::HashMap,
  fmt,
  sync::{Arc, Mutex},
};

use crate::{
  app::GlobalTrayIconEventListener,
  tray::{TrayIcon, TrayIconId},
  AppHandle, Icon, Runtime,
};

pub struct TrayManager<R: Runtime> {
  pub(crate) icon: Option<Icon>,
  /// Tray icons
  pub(crate) icons: Arc<Mutex<Vec<TrayIcon<R>>>>,
  /// Global Tray icon event listeners.
  pub(crate) global_event_listeners: Arc<Mutex<Vec<GlobalTrayIconEventListener<AppHandle<R>>>>>,
  /// Tray icon event listeners.
  pub(crate) event_listeners:
    Arc<Mutex<HashMap<TrayIconId, GlobalTrayIconEventListener<TrayIcon<R>>>>>,
}

impl<R: Runtime> fmt::Debug for TrayManager<R> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("TrayManager")
      .field("icon", &self.icon)
      .finish()
  }
}
