// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::PhysicalRect;
use tao::dpi::{LogicalPosition, LogicalSize, PhysicalPosition};

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect {
    use objc2_app_kit::NSScreen;
    use tao::platform::macos::MonitorHandleExtMacOS;
    if let Some(ns_screen) = self.ns_screen() {
      let ns_screen: &NSScreen = unsafe { &*ns_screen.cast() };
      let rect = ns_screen.visibleFrame();
      let scale_factor = self.scale_factor();
      PhysicalRect {
        size: LogicalSize::new(rect.size.width, rect.size.height).to_physical(scale_factor),
        position: LogicalPosition::new(rect.origin.x, rect.origin.y).to_physical(scale_factor),
      }
    } else {
      PhysicalRect {
        size: self.size(),
        position: PhysicalPosition::default(),
      }
    }
  }
}
