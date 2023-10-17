// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  command,
  plugin::{Builder, TauriPlugin},
  AppHandle, Runtime,
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
pub fn show<R: Runtime>(app: AppHandle<R>) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  app.show()?;
  Ok(())
}

#[command(root = "crate")]
#[allow(unused_variables)]
pub fn hide<R: Runtime>(app: AppHandle<R>) -> crate::Result<()> {
  #[cfg(target_os = "macos")]
  app.hide()?;
  Ok(())
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("app")
    .invoke_handler(crate::generate_handler![
      version,
      name,
      tauri_version,
      show,
      hide
    ])
    .build()
}
