// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::dpi::PhysicalRect;
use winit::monitor::MonitorHandle;

use crate::platform::{MonitorExt, monitor_bounds};

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    // TODO: implement native Linux/BSD work-area lookup via winit-gtk4 when it is available.
    monitor_bounds(self)
  }
}
