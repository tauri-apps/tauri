// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_utils::{config::BundleType, Theme};

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
pub fn identifier<R: Runtime>(app: AppHandle<R>) -> String {
  app.config().identifier.clone()
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
#[allow(unused_variables)]
pub async fn fetch_data_store_identifiers<R: Runtime>(
  app: AppHandle<R>,
) -> crate::Result<Vec<[u8; 16]>> {
  #[cfg(target_vendor = "apple")]
  return app.fetch_data_store_identifiers().await;
  #[cfg(not(target_vendor = "apple"))]
  return Ok(Vec::new());
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub async fn remove_data_store<R: Runtime>(app: AppHandle<R>, uuid: [u8; 16]) -> crate::Result<()> {
  #[cfg(target_vendor = "apple")]
  app.remove_data_store(uuid).await?;
  #[cfg(not(target_vendor = "apple"))]
  let _ = uuid;
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

#[command(root = "crate")]
pub async fn set_dock_visibility<R: Runtime>(
  app: AppHandle<R>,
  visible: bool,
) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  {
    let mut focused_window = None;
    for window in app.manager.windows().into_values() {
      if window.is_focused().unwrap_or_default() {
        focused_window.replace(window);
        break;
      }
    }

    app.set_dock_visibility(visible)?;

    // retain focus
    if let Some(focused_window) = focused_window {
      let _ = focused_window.set_focus();
    }
  }
  #[cfg(not(target_os = "macos"))]
  let (_app, _visible) = (app, visible);
  Ok(())
}

#[command(root = "crate")]
pub fn bundle_type() -> Option<BundleType> {
  tauri_utils::platform::bundle_type()
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("app")
    .invoke_handler(crate::generate_handler![
      #![plugin(app)]
      version,
      name,
      tauri_version,
      identifier,
      app_show,
      app_hide,
      fetch_data_store_identifiers,
      remove_data_store,
      default_window_icon,
      set_app_theme,
      set_dock_visibility,
      bundle_type,
    ])
    .setup(|_app, _api| {
      #[cfg(target_os = "android")]
      {
        let handle = _api.register_android_plugin("app.tauri", "AppPlugin")?;
        _app.manage(AppPlugin(handle));
      }
      Ok(())
    })
    .build()
}

#[cfg(target_os = "android")]
pub(crate) struct AppPlugin<R: Runtime>(pub crate::plugin::PluginHandle<R>);
