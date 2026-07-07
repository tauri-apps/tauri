// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::process::exit;

fn main() -> wry::Result<()> {
  use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
  };
  use wry::http::Request;
  use wry::{
    http::{header::CONTENT_TYPE, Response},
    WebViewBuilder,
  };

  let event_loop = EventLoop::new();
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let handler = |req: Request<String>| {
    if req.body() == "process-complete" {
      exit(0);
    }
  };

  let builder = WebViewBuilder::new()
    .with_custom_protocol("wrybench".into(), move |_id, request| {
      let path = request.uri().to_string();
      let requested_asset_path = path.strip_prefix("wrybench://localhost").unwrap();
      let (data, mimetype): (_, String) = match requested_asset_path {
        "/index.css" => (
          include_bytes!("static/index.css").as_slice().into(),
          "text/css".into(),
        ),
        "/site.js" => (
          include_bytes!("static/site.js").as_slice().into(),
          "text/javascript".into(),
        ),
        "/worker.js" => (
          include_bytes!("static/worker.js").as_slice().into(),
          "text/javascript".into(),
        ),
        _ => (
          include_bytes!("static/index.html").as_slice().into(),
          "text/html".into(),
        ),
      };

      Response::builder()
        .header(CONTENT_TYPE, mimetype)
        .body(data)
        .unwrap()
    })
    .with_url("wrybench://localhost")
    .with_ipc_handler(handler);

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

    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => *control_flow = ControlFlow::Exit,
      _ => {}
    }
  });
}
