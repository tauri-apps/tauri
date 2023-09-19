// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri::{
  plugin::{Builder, TauriPlugin},
  Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;
#[cfg(mobile)]
mod mobile;

mod error;
mod models;

#[cfg(desktop)]
use desktop::Sample;
#[cfg(mobile)]
use mobile::Sample;

pub use error::*;

/// Extensions to [`tauri::App`], [`tauri::AppHandle`] and [`tauri::Window`] to access the sample APIs.
pub trait SampleExt<R: Runtime> {
  fn sample(&self) -> &Sample<R>;
}

impl<R: Runtime, T: Manager<R>> crate::SampleExt<R> for T {
  fn sample(&self) -> &Sample<R> {
    self.state::<Sample<R>>().inner()
  }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
  Builder::new("sample")
    .setup(|app, api| {
      #[cfg(mobile)]
      let sample = mobile::init(app, api)?;
      #[cfg(desktop)]
      let sample = desktop::init(app, api)?;
      app.manage(sample);

      Ok(())
    })
    .on_navigation(|window, url| {
      println!("navigation {} {url}", window.label());
      true
    })
    .build()
}
