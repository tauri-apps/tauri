// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Example demonstrating the permission handler API.
//!
//! Run:  cargo run --example permission_handler
//! Then click the buttons and watch the terminal output.

fn main() -> wry::Result<()> {
  use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  use wry::{PermissionKind, PermissionResponse, WebViewBuilder};

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new()
    .with_title("Permission Handler Example")
    .with_inner_size(tao::dpi::LogicalSize::new(800, 600))
    .build(&event_loop)
    .unwrap();

  let builder = WebViewBuilder::new()
    .with_url("https://permission.site/")
    .with_permission_handler(|kind| {
      let response = match kind {
        PermissionKind::Geolocation => PermissionResponse::Default,
        _ => PermissionResponse::Allow,
      };
      println!("[permission] {kind} → {response}");
      response
    });

  #[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  ))]
  let _webview = builder.build(&window)?;
  #[cfg(not(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  )))]
  let _webview = {
    use tao::platform::unix::WindowExtUnix;
    use wry::WebViewBuilderExtUnix;
    let vbox = window.default_vbox().unwrap();
    builder.build_gtk(vbox)?
  };

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;
    if let Event::WindowEvent {
      event: WindowEvent::CloseRequested,
      ..
    } = event
    {
      *control_flow = ControlFlow::Exit;
    }
  });
}
