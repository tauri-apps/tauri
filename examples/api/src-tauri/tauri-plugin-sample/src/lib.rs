// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Deserialize;
use std::path::PathBuf;
use tauri::{
  Manager, Runtime, State,
  plugin::{Builder, TauriPlugin},
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
  fn sample(&self) -> State<'_, Sample<R>>;
}

impl<R: Runtime, T: Manager<R>> crate::SampleExt<R> for T {
  fn sample(&self) -> State<'_, Sample<R>> {
    self.state::<Sample<R>>()
  }
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct PingScope {
  path: PathBuf,
}

#[allow(dead_code)]
#[derive(Debug, Deserialize)]
struct SampleScope {
  path: PathBuf,
}

#[tauri::command]
fn ping<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
  value: Option<String>,
  scope: tauri::ipc::CommandScope<PingScope>,
  global_scope: tauri::ipc::GlobalScope<SampleScope>,
) -> std::result::Result<PingResponse, String> {
  println!("local scope {scope:?}");
  println!("global scope {global_scope:?}");
  app
    .sample()
    .ping(PingRequest {
      value,
      on_event: tauri::ipc::Channel::new(|_| Ok(())),
    })
    .map_err(|e| e.to_string())
}

/// Resolves `JSArray` and `JSObject` values from the native (Kotlin or Swift) plugin.
#[tauri::command]
fn js_values<R: tauri::Runtime>(
  app: tauri::AppHandle<R>,
) -> std::result::Result<serde_json::Value, String> {
  #[cfg(mobile)]
  return app.sample().js_values().map_err(|e| e.to_string());
  #[cfg(desktop)]
  {
    let _ = app;
    Err("js_values is only implemented on mobile".into())
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
    .invoke_handler(tauri::generate_handler![ping, js_values])
    .on_navigation(|window, url| {
      println!("navigation {} {url}", window.label());
      true
    })
    .build()
}
