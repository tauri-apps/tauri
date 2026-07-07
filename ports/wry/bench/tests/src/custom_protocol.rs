// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use std::process::exit;

const INDEX_HTML: &[u8] = br#"
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8" />
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
</head>
<body>
    <h1>Welcome to WRY!</h1>
    <script>
    document.addEventListener('DOMContentLoaded', () => {
        ipc.postMessage('dom-loaded')
    })
    </script>
</body>
</html>"#;

#[derive(Debug, Serialize, Deserialize)]
struct MessageParameters {
  message: String,
}

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
    if req.body() == "dom-loaded" {
      exit(0);
    }
  };

  let builder = WebViewBuilder::new()
    .with_ipc_handler(handler)
    .with_custom_protocol("wrybench".into(), move |_id, _request| {
      Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(INDEX_HTML.into())
        .unwrap()
    })
    .with_url("wrybench://localhost");

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
  })
}
