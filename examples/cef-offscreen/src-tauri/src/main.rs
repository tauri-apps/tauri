// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod renderer;

use std::sync::{Arc, Mutex};
use tauri::{Manager, WebviewUrl, WindowEvent};
use tauri_runtime_cef::{Cef, OffscreenSurface, WebviewWindowBuilderCefExt, WindowCefExt};

#[tauri::command]
fn greet() -> &'static str {
  "Hello from Rust"
}

#[tauri_runtime_cef::cef_entry_point]
fn main() {
  tauri::Builder::default()
    .runtime(Cef::default().with_settings(|settings| settings.windowless_rendering_enabled = 1))
    .invoke_handler(tauri::generate_handler![greet])
    .setup(|app| {
      let handle = app.handle().clone();
      let surface = OffscreenSurface::new(move || {
        if let Some(window) = handle.get_webview_window("main") {
          let _ = window.request_redraw();
        }
      });
      let webview =
        tauri::WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
          .title("CEF + native Vello")
          .inner_size(960.0, 640.0)
          .offscreen(surface.clone())
          .with_browser_settings(|settings| settings.windowless_frame_rate = 60)
          .build()?;
      let window = Arc::new(webview.as_ref().window());
      let renderer = Arc::new(Mutex::new(Some(pollster::block_on(
        renderer::Renderer::new(window.clone()),
      )?)));
      let render = renderer.clone();
      let redraw_window = window.clone();
      window.on_redraw(move || {
        let Ok(size) = redraw_window.inner_size() else {
          return;
        };
        let mut renderer = render.lock().unwrap();
        let Some(renderer) = renderer.as_mut() else {
          return;
        };
        if let Err(error) = renderer.draw(size.width, size.height, surface.snapshot()) {
          eprintln!("failed to present: {error}");
        }
      })?;
      window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
          // The GPU surface must be released before Tauri destroys the native window.
          renderer.lock().unwrap().take();
        }
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("failed to run the offscreen example");
}
