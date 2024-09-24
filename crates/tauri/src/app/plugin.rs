// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_utils::Theme;

use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Manager, ResourceId, Runtime, Webview,
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

#[command(root = "crate")]
pub fn default_window_icon<R: Runtime>(
  webview: Webview<R>,
  app: AppHandle<R>,
) -> Option<ResourceId> {
  app.default_window_icon().cloned().map(|icon| {
    let mut resources_table = webview.resources_table();
    resources_table.add(icon.to_owned())
  })
}

#[command(root = "crate")]
pub async fn set_app_theme<R: Runtime>(app: AppHandle<R>, theme: Option<Theme>) {
  app.set_theme(theme);
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
      set_app_theme,
    ])
    .build()
}
