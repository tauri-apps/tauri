// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tao::{
  event::{ElementState, Event, KeyEvent, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  keyboard::Key,
  window::WindowBuilder,
};
use wry::WebViewBuilder;

#[cfg(target_os = "macos")]
use {objc2_app_kit::NSWindow, tao::platform::macos::WindowExtMacOS, wry::WebViewExtMacOS};
#[cfg(target_os = "windows")]
use {tao::platform::windows::WindowExtWindows, wry::WebViewExtWindows};

#[cfg(not(any(
  target_os = "windows",
  target_os = "macos",
  target_os = "ios",
  target_os = "android"
)))]
#[cfg(not(any(
  target_os = "windows",
  target_os = "macos",
  target_os = "ios",
  target_os = "android"
)))]
use {
  tao::platform::unix::WindowExtUnix,
  wry::{WebViewBuilderExtUnix, WebViewExtUnix},
};

fn main() -> wry::Result<()> {
  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();
  let window2 = WindowBuilder::new().build(&event_loop).unwrap();

  let builder = WebViewBuilder::new().with_url("https://tauri.app");

  #[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  ))]
  let webview = builder.build(&window)?;
  #[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  )))]
  let mut webview = {
    use tao::platform::unix::WindowExtUnix;
    let vbox = window.default_vbox().unwrap();
    builder.build_gtk(vbox)?
  };

  let mut webview_container = window.id();

  event_loop.run(move |event, _event_loop, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,

      Event::WindowEvent {
        event:
          WindowEvent::KeyboardInput {
            event:
              KeyEvent {
                logical_key: Key::Character("x"),
                state: ElementState::Pressed,
                ..
              },
            ..
          },
        ..
      } => {
        let new_parent = if webview_container == window.id() {
          &window2
        } else {
          &window
        };
        webview_container = new_parent.id();

        #[cfg(target_os = "macos")]
        webview
          .reparent(new_parent.ns_window() as *mut NSWindow)
          .unwrap();
        #[cfg(not(any(
          target_os = "windows",
          target_os = "macos",
          target_os = "ios",
          target_os = "android"
        )))]
        webview
          .reparent(new_parent.default_vbox().unwrap())
          .unwrap();
        #[cfg(target_os = "windows")]
        webview.reparent(new_parent.hwnd()).unwrap();
      }
      _ => {}
    }
  });
}
