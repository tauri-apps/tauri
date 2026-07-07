// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::collections::HashMap;
use tao::{
  event::{Event, WindowEvent},
  event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy, EventLoopWindowTarget},
  window::{Window, WindowBuilder, WindowId},
};
use wry::{http::Request, WebView, WebViewBuilder};

enum UserEvent {
  CloseWindow(WindowId),
  NewTitle(WindowId, String),
  NewWindow,
}

fn main() -> wry::Result<()> {
  let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
  let mut webviews = HashMap::new();
  let proxy = event_loop.create_proxy();

  let (window, webview) = create_new_window(
    format!("Window {}", webviews.len() + 1),
    &event_loop,
    proxy.clone(),
  );
  webviews.insert(window.id(), (window, webview));

  event_loop.run(move |event, event_loop, control_flow| {
    *control_flow = ControlFlow::Wait;

    match event {
      Event::WindowEvent {
        event: WindowEvent::CloseRequested,
        window_id,
        ..
      } => {
        webviews.remove(&window_id);
        if webviews.is_empty() {
          *control_flow = ControlFlow::Exit
        }
      }
      Event::UserEvent(UserEvent::NewWindow) => {
        let (window, webview) = create_new_window(
          format!("Window {}", webviews.len() + 1),
          event_loop,
          proxy.clone(),
        );
        webviews.insert(window.id(), (window, webview));
      }
      Event::UserEvent(UserEvent::CloseWindow(id)) => {
        webviews.remove(&id);
        if webviews.is_empty() {
          *control_flow = ControlFlow::Exit
        }
      }

      Event::UserEvent(UserEvent::NewTitle(id, title)) => {
        webviews.get(&id).unwrap().0.set_title(&title);
      }
      _ => (),
    }
  });
}

fn create_new_window(
  title: String,
  event_loop: &EventLoopWindowTarget<UserEvent>,
  proxy: EventLoopProxy<UserEvent>,
) -> (Window, WebView) {
  let window = WindowBuilder::new()
    .with_title(title)
    .build(event_loop)
    .unwrap();
  let window_id = window.id();
  let handler = move |req: Request<String>| {
    let body = req.body();
    match body.as_str() {
      "new-window" => {
        let _ = proxy.send_event(UserEvent::NewWindow);
      }
      "close" => {
        let _ = proxy.send_event(UserEvent::CloseWindow(window_id));
      }
      _ if body.starts_with("change-title") => {
        let title = body.replace("change-title:", "");
        let _ = proxy.send_event(UserEvent::NewTitle(window_id, title));
      }
      _ => {}
    }
  };

  let builder = WebViewBuilder::new()
    .with_html(
      r#"
        <button onclick="window.ipc.postMessage('new-window')">Open a new window</button>
        <button onclick="window.ipc.postMessage('close')">Close current window</button>
        <input oninput="window.ipc.postMessage(`change-title:${this.value}`)" />
    "#,
    )
    .with_ipc_handler(handler);

  #[cfg(any(
    target_os = "windows",
    target_os = "macos",
    target_os = "ios",
    target_os = "android"
  ))]
  let webview = builder.build(&window).unwrap();
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
    builder.build_gtk(vbox).unwrap()
  };
  (window, webview)
}
