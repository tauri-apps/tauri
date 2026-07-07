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
  let window = WindowBuilder::new().build(&event_loop).unwrap();

  let builder = WebViewBuilder::new().with_url("https://www.httpbin.org/cookies/set?foo=bar");

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

  webview.set_cookie(
    cookie::Cookie::build(("foo1", "bar1"))
      .domain("www.httpbin.org")
      .path("/")
      .secure(true)
      .http_only(true)
      .max_age(cookie::time::Duration::seconds(10))
      .inner(),
  )?;

  let cookie_deleted = cookie::Cookie::build(("will_be_deleted", "will_be_deleted"));

  webview.set_cookie(cookie_deleted.inner())?;
  println!("Setting Cookies:");
  for cookie in webview.cookies()? {
    println!("\t{cookie}");
  }

  println!("After Deleting:");
  webview.delete_cookie(cookie_deleted.inner())?;
  for cookie in webview.cookies()? {
    println!("\t{cookie}");
  }

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
