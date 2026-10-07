// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod renderer;

use std::sync::{
  Arc, Mutex,
  atomic::{AtomicUsize, Ordering},
};

use tauri::WebviewUrl;
use tauri_runtime_cef::{
  Cef, NativeEventResponse, Offscreen, OffscreenEvent, WebviewWindowBuilderCefExt,
  cef::{PaintElementType, Rect},
  winit::event::{ElementState, WindowEvent},
};

#[derive(Clone)]
struct Frame {
  width: u32,
  height: u32,
  pixels: Arc<[u8]>,
  serial: u64,
}

#[derive(Clone)]
struct Frames {
  view: Option<Frame>,
  popup: Option<Frame>,
  popup_rect: Option<Rect>,
  visible: bool,
  serial: u64,
}

impl Default for Frames {
  fn default() -> Self {
    Self {
      view: None,
      popup: None,
      popup_rect: None,
      visible: true,
      serial: 0,
    }
  }
}

#[derive(Default)]
struct State {
  frames: Mutex<Frames>,
  renderer: Mutex<Option<renderer::Renderer>>,
  native_clicks: AtomicUsize,
}

#[tauri::command]
fn greet() -> &'static str {
  "Hello from Rust"
}

#[tauri_runtime_cef::cef_entry_point]
fn main() {
  let state = Arc::new(State::default());
  let events = state.clone();
  let runtime = Cef::default()
    .with_settings(|settings| settings.windowless_rendering_enabled = 1)
    .on_window_event(move |_, _, window, event| {
      match event {
        WindowEvent::RedrawRequested => {
          let size = window.surface_size();
          if size.width == 0 || size.height == 0 {
            return NativeEventResponse::Continue;
          }
          let frames = events.frames.lock().unwrap().clone();
          let mut renderer = events.renderer.lock().unwrap();
          if renderer.is_none() {
            *renderer = Some(
              pollster::block_on(renderer::Renderer::new(window.clone()))
                .expect("create native renderer"),
            );
          }
          renderer
            .as_mut()
            .unwrap()
            .draw(
              size.width,
              size.height,
              window.scale_factor(),
              &frames,
              events.native_clicks.load(Ordering::Relaxed),
            )
            .expect("present CEF over the native scene");
        }
        WindowEvent::Destroyed => {
          // The surface retains the native window. Release it before window teardown.
          events.renderer.lock().unwrap().take();
        }
        WindowEvent::PointerMoved { position, .. }
        | WindowEvent::PointerButton { position, .. }
          if position.x / window.scale_factor() >= 430.0 =>
        {
          window.set_cursor_visible(true);
          window.set_cursor(tauri_runtime_cef::winit::cursor::CursorIcon::Pointer.into());
          if matches!(
            event,
            WindowEvent::PointerButton {
              state: ElementState::Pressed,
              primary: true,
              ..
            }
          ) {
            let count = events.native_clicks.fetch_add(1, Ordering::Relaxed) + 1;
            window.set_title(&format!("CEF Offscreen — native clicks: {count}"));
            window.request_redraw();
          }
          // This region belongs to the native scene. Browser drags that began
          // in the HTML panel still finish through Tauri's pointer capture.
          return NativeEventResponse::Handled;
        }
        _ => {}
      }
      NativeEventResponse::Continue
    });

  tauri::Builder::default()
    .runtime(runtime)
    .invoke_handler(tauri::generate_handler![greet])
    .setup(move |app| {
      let options = Offscreen::new(move |_, event| {
        let mut frames = state.frames.lock().unwrap();
        match event {
          OffscreenEvent::Paint {
            element,
            pixels,
            size,
            ..
          } => {
            // CEF only lends the pixels during this callback. This example keeps
            // the latest complete image for the next native redraw; an application
            // can instead upload dirty regions or import accelerated paint here.
            frames.serial += 1;
            let frame = Frame {
              width: size.width,
              height: size.height,
              pixels: Arc::from(pixels),
              serial: frames.serial,
            };
            if element == PaintElementType::POPUP {
              frames.popup = Some(frame);
            } else {
              frames.view = Some(frame);
            }
          }
          OffscreenEvent::Popup(rect) => {
            frames.popup_rect = rect;
            if frames.popup_rect.is_none() {
              frames.popup = None;
            }
          }
          OffscreenEvent::Visibility(visible) => frames.visible = visible,
          OffscreenEvent::Closed => {
            *frames = Frames {
              visible: false,
              ..Default::default()
            };
          }
          _ => {}
        }
      });
      tauri::WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
        .title("CEF Offscreen")
        .inner_size(960.0, 640.0)
        .min_inner_size(700.0, 560.0)
        .offscreen(options)
        .with_browser_settings(|settings| settings.windowless_frame_rate = 60)
        .build()?;
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("run CEF offscreen example");
}
