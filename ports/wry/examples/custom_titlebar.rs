// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tao::{
  dpi::PhysicalSize,
  event::{Event, StartCause, WindowEvent},
  event_loop::{ControlFlow, EventLoopBuilder},
  window::{CursorIcon, ResizeDirection, Window, WindowBuilder},
};
use wry::{http::Request, WebViewBuilder};

#[derive(Debug)]
enum HitTestResult {
  Client,
  Left,
  Right,
  Top,
  Bottom,
  TopLeft,
  TopRight,
  BottomLeft,
  BottomRight,
  NoWhere,
}

impl HitTestResult {
  fn drag_resize_window(&self, window: &Window) {
    let _ = window.drag_resize_window(match self {
      HitTestResult::Left => ResizeDirection::West,
      HitTestResult::Right => ResizeDirection::East,
      HitTestResult::Top => ResizeDirection::North,
      HitTestResult::Bottom => ResizeDirection::South,
      HitTestResult::TopLeft => ResizeDirection::NorthWest,
      HitTestResult::TopRight => ResizeDirection::NorthEast,
      HitTestResult::BottomLeft => ResizeDirection::SouthWest,
      HitTestResult::BottomRight => ResizeDirection::SouthEast,
      _ => unreachable!(),
    });
  }

  fn change_cursor(&self, window: &Window) {
    window.set_cursor_icon(match self {
      HitTestResult::Left => CursorIcon::WResize,
      HitTestResult::Right => CursorIcon::EResize,
      HitTestResult::Top => CursorIcon::NResize,
      HitTestResult::Bottom => CursorIcon::SResize,
      HitTestResult::TopLeft => CursorIcon::NwResize,
      HitTestResult::TopRight => CursorIcon::NeResize,
      HitTestResult::BottomLeft => CursorIcon::SwResize,
      HitTestResult::BottomRight => CursorIcon::SeResize,
      _ => CursorIcon::Default,
    });
  }
}

fn hit_test(window_size: PhysicalSize<u32>, x: i32, y: i32, scale: f64) -> HitTestResult {
  const BORDERLESS_RESIZE_INSET: f64 = 5.0;

  const CLIENT: isize = 0b0000;
  const LEFT: isize = 0b0001;
  const RIGHT: isize = 0b0010;
  const TOP: isize = 0b0100;
  const BOTTOM: isize = 0b1000;
  const TOPLEFT: isize = TOP | LEFT;
  const TOPRIGHT: isize = TOP | RIGHT;
  const BOTTOMLEFT: isize = BOTTOM | LEFT;
  const BOTTOMRIGHT: isize = BOTTOM | RIGHT;

  let top = 0;
  let left = 0;
  let bottom = top + window_size.height as i32;
  let right = left + window_size.width as i32;

  let x = x * scale as i32;
  let y = y * scale as i32;

  let inset = (BORDERLESS_RESIZE_INSET * scale) as i32;

  #[rustfmt::skip]
      let result =
          (LEFT * (if x < (left + inset) { 1 } else { 0 }))
        | (RIGHT * (if x >= (right - inset) { 1 } else { 0 }))
        | (TOP * (if y < (top + inset) { 1 } else { 0 }))
        | (BOTTOM * (if y >= (bottom - inset) { 1 } else { 0 }));

  match result {
    CLIENT => HitTestResult::Client,
    LEFT => HitTestResult::Left,
    RIGHT => HitTestResult::Right,
    TOP => HitTestResult::Top,
    BOTTOM => HitTestResult::Bottom,
    TOPLEFT => HitTestResult::TopLeft,
    TOPRIGHT => HitTestResult::TopRight,
    BOTTOMLEFT => HitTestResult::BottomLeft,
    BOTTOMRIGHT => HitTestResult::BottomRight,
    _ => HitTestResult::NoWhere,
  }
}

enum UserEvent {
  Minimize,
  Maximize,
  DragWindow,
  CloseWindow,
  MouseDown(i32, i32),
  MouseMove(i32, i32),
}

fn main() -> wry::Result<()> {
  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
  let window = WindowBuilder::new()
    .with_decorations(false)
    .build(&event_loop)
    .unwrap();

  const HTML: &str = r#"
  <html>

  <head>
      <style>
          html {
            font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
          }

          * {
              padding: 0;
              margin: 0;
              box-sizing: border-box;
          }

          *[data-wry-darg-region] {
            app-region: drag; /* Supported on Windows only, on WebView2 123+ and makes dragging with touch work */
          }

          main {
            display: grid;
            place-items: center;
            height: calc(100vh - 30px);
          }

          .titlebar {
              height: 30px;
              padding-left: 5px;
              display: grid;
              grid-auto-flow: column;
              grid-template-columns: 1fr max-content max-content max-content;
              align-items: center;
              background: #1F1F1F;
              color: white;
              user-select: none;
          }

          .titlebar-button {
              display: inline-flex;
              justify-content: center;
              align-items: center;
              width: 30px;
              height: 30px;
          }

          .titlebar-button:hover {
              background: #3b3b3b;
          }

          .titlebar-button#close:hover {
              background: #da3d3d;
          }

          .titlebar-button img {
              filter: invert(100%);
          }
      </style>
  </head>

  <body>
      <div class="titlebar">
          <div data-wry-darg-region>Custom Titlebar</div>
          <div>
              <div class="titlebar-button" onclick="window.ipc.postMessage('minimize')">
                  <img src="https://api.iconify.design/codicon:chrome-minimize.svg" />
              </div>
              <div class="titlebar-button" onclick="window.ipc.postMessage('maximize')">
                  <img src="https://api.iconify.design/codicon:chrome-maximize.svg" />
              </div>
              <div class="titlebar-button" id="close" onclick="window.ipc.postMessage('close')">
                  <img src="https://api.iconify.design/codicon:close.svg" />
              </div>
          </div>
      </div>
      <main>
          <h4> WRYYYYYYYYYYYYYYYYYYYYYY! </h4>
      </main>
      <script>
          document.addEventListener('mousemove', (e) => window.ipc.postMessage(`mousemove:${e.clientX},${e.clientY}`))
          document.addEventListener('mousedown', (e) => {
              if (e.target.hasAttribute('data-wry-darg-region') && e.button === 0) {
                  e.detail === 2
                      ? window.ipc.postMessage('maximize')
                      : window.ipc.postMessage('drag_window');
              } else {
                window.ipc.postMessage(`mousedown:${e.clientX},${e.clientY}`);
              }
          })
          document.addEventListener('touchstart', (e) => {
              if (e.target.hasAttribute('data-wry-darg-region')) {
                  window.ipc.postMessage('drag_window');
              }
          })
      </script>
  </body>

  </html>
"#;

  let proxy = event_loop.create_proxy();
  let handler = move |req: Request<String>| {
    let body = req.body();
    let mut req = body.split([':', ',']);
    match req.next().unwrap() {
      "minimize" => {
        let _ = proxy.send_event(UserEvent::Minimize);
      }
      "maximize" => {
        let _ = proxy.send_event(UserEvent::Maximize);
      }
      "drag_window" => {
        let _ = proxy.send_event(UserEvent::DragWindow);
      }
      "close" => {
        let _ = proxy.send_event(UserEvent::CloseWindow);
      }
      "mousedown" => {
        let x = req.next().unwrap().parse().unwrap();
        let y = req.next().unwrap().parse().unwrap();
        let _ = proxy.send_event(UserEvent::MouseDown(x, y));
      }
      "mousemove" => {
        let x = req.next().unwrap().parse().unwrap();
        let y = req.next().unwrap().parse().unwrap();
        let _ = proxy.send_event(UserEvent::MouseMove(x, y));
      }
      _ => {}
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

  let mut webview: Option<wry::WebView> = Some(webview);

  event_loop.run(move |event, _, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::NewEvents(StartCause::Init) => println!("Wry application started!"),
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        ..
      }
      | Event::UserEvent(UserEvent::CloseWindow) => {
        let _ = webview.take();
        *control_flow = ControlFlow::Exit
      }

      Event::UserEvent(e) => match e {
        UserEvent::Minimize => window.set_minimized(true),
        UserEvent::Maximize => window.set_maximized(!window.is_maximized()),
        UserEvent::DragWindow => window.drag_window().unwrap(),
        UserEvent::MouseDown(x, y) => {
          let res = hit_test(window.inner_size(), x, y, window.scale_factor());
          match res {
            HitTestResult::Client | HitTestResult::NoWhere => {}
            _ => res.drag_resize_window(&window),
          }
        }
        UserEvent::MouseMove(x, y) => {
          hit_test(window.inner_size(), x, y, window.scale_factor()).change_cursor(&window);
        }
        UserEvent::CloseWindow => { /* handled above */ }
      },
      _ => (),
    }
  });
}
