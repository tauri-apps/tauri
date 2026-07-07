// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

#[allow(dead_code)]
fn needs_sync<T: Sync>() {}

#[test]
fn window_sync() {
  // ensures that `Window` implements `Sync`
  needs_sync::<tao::window::Window>();
}
