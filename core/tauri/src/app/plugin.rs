// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Manager, ResourceId, Runtime,
};

#[command(root = "crate")]
pub fn version<R: Runtime>(app: AppHandle<R>) -> String {
  app.package_info().version.to_string()
}

#[command(root = "crate")]
pub fn name<R: Runtime>(app: AppHandle<R>) -> String {
  app.package_info().name.clone()
}

#[command(root = "crate")]
pub fn tauri_version() -> &'static str {
  crate::VERSION
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub fn app_show<R: Runtime>(app: AppHandle<R>) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  app.show()?;
  Ok(())
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub fn app_hide<R: Runtime>(app: AppHandle<R>) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  app.hide()?;
  Ok(())
}

struct DefaultWindowIconRid(ResourceId);

#[command(root = "crate")]
pub fn default_window_icon<R: Runtime>(app: AppHandle<R>) -> Option<ResourceId> {
  app
    .try_state::<DefaultWindowIconRid>()
    .map(|rid| rid.0)
    .or_else(|| {
      app.default_window_icon().cloned().map(|icon| {
        let mut resources_table = app.resources_table();
        let rid = resources_table.add(icon.to_owned());
        app.manage(DefaultWindowIconRid(rid));
        rid
      })
    })
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("app")
    .invoke_handler(crate::generate_handler![
      version,
      name,
      tauri_version,
      app_show,
      app_hide,
      default_window_icon,
    ])
    .build()
}
