// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc2_app_kit::NSScreen;
use tauri_runtime::dpi::{LogicalSize, PhysicalRect};
use winit::{monitor::MonitorHandle, platform::macos::MonitorHandleExtMacOS};

use crate::platform::{MonitorExt, monitor_bounds};

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let Some(ns_screen) = self.ns_screen() else {
      return monitor_bounds(self);
    };

    let ns_screen: &NSScreen = unsafe { &*ns_screen.cast() };
    let screen_frame = ns_screen.frame();
    let visible_frame = ns_screen.visibleFrame();
    let scale_factor = self.scale_factor();

    let position = self.position().unwrap_or_default();
    let mut position = position.to_logical::<f64>(scale_factor);
    position.x += visible_frame.origin.x - screen_frame.origin.x;
    position.y += (screen_frame.origin.y + screen_frame.size.height)
      - (visible_frame.origin.y + visible_frame.size.height);

    let size = LogicalSize::new(visible_frame.size.width, visible_frame.size.height);

    PhysicalRect {
      position: position.to_physical(scale_factor),
      size: size.to_physical(scale_factor),
    }
  }
}
