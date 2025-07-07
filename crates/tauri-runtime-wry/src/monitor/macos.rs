// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::dpi::{LogicalSize, PhysicalRect};

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    use objc2_app_kit::NSScreen;
    use tao::platform::macos::MonitorHandleExtMacOS;
    if let Some(ns_screen) = self.ns_screen() {
      let ns_screen: &NSScreen = unsafe { &*ns_screen.cast() };
      let screen_frame = ns_screen.frame();
      let visible_frame = ns_screen.visibleFrame();

      let scale_factor = self.scale_factor();

      let mut position = self.position().to_logical::<f64>(scale_factor);

      position.x += visible_frame.origin.x - screen_frame.origin.x;

      PhysicalRect {
        size: LogicalSize::new(visible_frame.size.width, visible_frame.size.height)
          .to_physical(scale_factor),
        position: position.to_physical(scale_factor),
      }
    } else {
      PhysicalRect {
        size: self.size(),
        position: self.position(),
      }
    }
  }
}
