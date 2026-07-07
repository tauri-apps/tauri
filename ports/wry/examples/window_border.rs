// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use dpi::LogicalSize;
use tao::{
  event::{Event, StartCause, WindowEvent},
  event_loop::{ControlFlow, EventLoopBuilder},
  window::WindowBuilder,
};
use wry::{http::Request, WebViewBuilder};

#[derive(Debug)]
enum UserEvent {
  TogglShadows,
}

fn main() -> wry::Result<()> {
  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
  let window = WindowBuilder::new()
    .with_inner_size(LogicalSize::new(500, 500))
    .with_decorations(false)
    .build(&event_loop)
    .unwrap();

  const HTML: &str = r#"
  <html>

  <head>
      <style>
          html {
            font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
            width: 100vw;
            height: 100vh;
            background-color: #1f1f1f;
            border: 1px solid rgb(148, 231, 155);
          }

          * {
              padding: 0;
              margin: 0;
              box-sizing: border-box;
          }
      </style>
  </head>

  <body>
    <p>
      Click the window to toggle shadows.
    </p>

    <script>
      window.addEventListener('click', () => window.ipc.postMessage('toggleShadows'))
    </script>
  </body>

  </html>
"#;

  let proxy = event_loop.create_proxy();
  let handler = move |req: Request<String>| {
    if req.body().as_str() == "toggleShadows" {
      proxy.send_event(UserEvent::TogglShadows).unwrap();
    }
  };

  let builder = WebViewBuilder::new()
    .with_html(HTML)
    .with_ipc_handler(handler)
    .with_accept_first_mouse(true);

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
  let webview = {
    use tao::platform::unix::WindowExtUnix;
    use wry::WebViewBuilderExtUnix;
    let vbox = window.default_vbox().unwrap();
    builder.build_gtk(vbox)?
  };

  let mut webview = Some(webview);

  let mut shadow = true;

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::NewEvents(StartCause::Init) => println!("Wry application started!"),
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      } => {
        let _ = webview.take();
        *control_flow = ControlFlow::Exit
      }

      Event::UserEvent(e) => match e {
        UserEvent::TogglShadows => {
          shadow = !shadow;
          #[cfg(windows)]
          {
            use tao::platform::windows::WindowExtWindows;
            window.set_undecorated_shadow(shadow);
          }
        }
      },
      _ => (),
    }
  });
}
