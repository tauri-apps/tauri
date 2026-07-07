// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use winit::{
  application::ApplicationHandler,
  event::WindowEvent,
  event_loop::{ActiveEventLoop, EventLoop},
  window::{Window, WindowId},
};
use wry::{
  dpi::{LogicalPosition, LogicalSize},
  Rect, WebViewBuilder,
};

#[derive(Default)]
struct State {
  window: Option<Window>,
  webview1: Option<wry::WebView>,
  webview2: Option<wry::WebView>,
  webview3: Option<wry::WebView>,
  webview4: Option<wry::WebView>,
}

impl ApplicationHandler for State {
  fn resumed(&mut self, event_loop: &ActiveEventLoop) {
    let mut attributes = Window::default_attributes();
    attributes.inner_size = Some(LogicalSize::new(800, 800).into());
    let window = event_loop.create_window(attributes).unwrap();

    let size = window.inner_size().to_logical::<u32>(window.scale_factor());

    let webview1 = WebViewBuilder::new()
      .with_bounds(Rect {
        position: LogicalPosition::new(0, 0).into(),
        size: LogicalSize::new(size.width / 2, size.height / 2).into(),
      })
      .with_url("https://tauri.app")
      .build_as_child(&window)
      .unwrap();

    let webview2 = WebViewBuilder::new()
      .with_bounds(Rect {
        position: LogicalPosition::new(size.width / 2, 0).into(),
        size: LogicalSize::new(size.width / 2, size.height / 2).into(),
      })
      .with_url("https://github.com/tauri-apps/wry")
      .build_as_child(&window)
      .unwrap();

    let webview3 = WebViewBuilder::new()
      .with_bounds(Rect {
        position: LogicalPosition::new(0, size.height / 2).into(),
        size: LogicalSize::new(size.width / 2, size.height / 2).into(),
      })
      .with_url("https://twitter.com/TauriApps")
      .build_as_child(&window)
      .unwrap();

    let webview4 = WebViewBuilder::new()
      .with_bounds(Rect {
        position: LogicalPosition::new(size.width / 2, size.height / 2).into(),
        size: LogicalSize::new(size.width / 2, size.height / 2).into(),
      })
      .with_url("https://google.com")
      .build_as_child(&window)
      .unwrap();

    self.window = Some(window);
    self.webview1 = Some(webview1);
    self.webview2 = Some(webview2);
    self.webview3 = Some(webview3);
    self.webview4 = Some(webview4);
  }

  fn window_event(
    &mut self,
    _event_loop: &ActiveEventLoop,
    _window_id: WindowId,
    event: WindowEvent,
  ) {
    match event {
      WindowEvent::Resized(size) => {
        if let (Some(window), Some(webview1), Some(webview2), Some(webview3), Some(webview4)) = (
          &self.window,
          &self.webview1,
          &self.webview2,
          &self.webview3,
          &self.webview4,
        ) {
          let size = size.to_logical::<u32>(window.scale_factor());

          webview1
            .set_bounds(Rect {
              position: LogicalPosition::new(0, 0).into(),
              size: LogicalSize::new(size.width / 2, size.height / 2).into(),
            })
            .unwrap();

          webview2
            .set_bounds(Rect {
              position: LogicalPosition::new(size.width / 2, 0).into(),
              size: LogicalSize::new(size.width / 2, size.height / 2).into(),
            })
            .unwrap();

          webview3
            .set_bounds(Rect {
              position: LogicalPosition::new(0, size.height / 2).into(),
              size: LogicalSize::new(size.width / 2, size.height / 2).into(),
            })
            .unwrap();

          webview4
            .set_bounds(Rect {
              position: LogicalPosition::new(size.width / 2, size.height / 2).into(),
              size: LogicalSize::new(size.width / 2, size.height / 2).into(),
            })
            .unwrap();
        }
      }
      WindowEvent::CloseRequested => {
        std::process::exit(0);
      }
      _ => {}
    }
  }

  fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd",
    ))]
    {
      let context = gtk::glib::MainContext::default();
      while context.pending() {
        context.iteration(false);
      }
    }
  }
}

fn main() -> wry::Result<()> {
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd",
  ))]
  {
    use gtk::prelude::DisplayExtManual;

    gtk::init()?;
    if gtk::gdk::Display::default().unwrap().backend().is_wayland() {
      panic!("This example doesn't support wayland!");
    }

    winit::platform::x11::register_xlib_error_hook(Box::new(|_display, error| {
      let error = error as *mut x11_dl::xlib::XErrorEvent;
      (unsafe { (*error).error_code }) == 170
    }));
  }

  let event_loop = EventLoop::new().unwrap();
  let mut state = State::default();
  event_loop.run_app(&mut state).unwrap();

  Ok(())
}
