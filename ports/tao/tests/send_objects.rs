// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
fn needs_send<T: Send>() {}

#[test]
fn event_loop_proxy_send() {
  #[allow(dead_code)]
  fn is_send<T: 'static + Send>() {
    // ensures that `EventLoopProxy` implements `Send`
    needs_send::<tao::event_loop::EventLoopProxy<T>>();
  }
}

#[test]
fn window_send() {
  // ensures that `Window` implements `Send`
  needs_send::<tao::window::Window>();
}

#[test]
fn ids_send() {
  // ensures that the various `..Id` types implement `Send`
  needs_send::<tao::window::WindowId>();
  needs_send::<tao::event::DeviceId>();
  needs_send::<tao::monitor::MonitorHandle>();
}
