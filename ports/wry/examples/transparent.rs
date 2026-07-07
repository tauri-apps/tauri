// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tao::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoop},
  window::WindowBuilder,
};
use wry::WebViewBuilder;

fn main() -> wry::Result<()> {
  let event_loop = EventLoop::new();
  #[allow(unused_mut)]
  let mut builder = WindowBuilder::new()
    .with_decorations(false)
    // There are actually three layer of background color when creating webview window.
    // The first is window background...
    .with_transparent(true);
  #[cfg(target_os = "windows")]
  {
    use tao::platform::windows::WindowBuilderExtWindows;
    builder = builder.with_undecorated_shadow(false);
  }
  let window = builder.build(&event_loop).unwrap();

  #[cfg(target_os = "windows")]
  {
    use tao::platform::windows::WindowExtWindows;
    window.set_undecorated_shadow(true);
  }

  let builder = WebViewBuilder::new()
    // The second is on webview...
    // Feature `transparent` is required for transparency to work.
    .with_transparent(true)
    // And the last is in html.
    .with_html(
      r#"<html>
          <body style="background-color:rgba(87,87,87,0.5);"></body>
          <script>
            window.onload = function() {
              document.body.innerText = `hello, ${navigator.userAgent}`;
            };
          </script>
        </html>"#,
    );

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
      *control_flow = ControlFlow::Exit
    }
  });
}
