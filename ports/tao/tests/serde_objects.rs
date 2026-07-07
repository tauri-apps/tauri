// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

#![cfg(feature = "serde")]

use serde::{Deserialize, Serialize};
use tao::{
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
  event::{ElementState, MouseButton, MouseScrollDelta, TouchPhase},
  keyboard::{Key, KeyCode, KeyLocation, ModifiersState},
  window::CursorIcon,
};

#[allow(dead_code)]
fn needs_serde<S: Serialize + Deserialize<'static>>() {}

#[test]
fn window_serde() {
  needs_serde::<CursorIcon>();
}

#[test]
fn events_serde() {
  needs_serde::<TouchPhase>();
  needs_serde::<ElementState>();
  needs_serde::<MouseButton>();
  needs_serde::<MouseScrollDelta>();
  needs_serde::<Key>();
  needs_serde::<KeyCode>();
  needs_serde::<KeyLocation>();
  needs_serde::<ModifiersState>();
}

#[test]
fn dpi_serde() {
  needs_serde::<LogicalPosition<f64>>();
  needs_serde::<PhysicalPosition<i32>>();
  needs_serde::<PhysicalPosition<f64>>();
  needs_serde::<LogicalSize<f64>>();
  needs_serde::<PhysicalSize<u32>>();
}
