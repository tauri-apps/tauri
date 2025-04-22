// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use gtk::prelude::MonitorExt;
use tao::platform::unix::MonitorHandleExtUnix;
use tauri_runtime::dpi::{PhysicalPosition, PhysicalRect, PhysicalSize};

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let rect = self.gdk_monitor().workarea();
    PhysicalRect {
      size: PhysicalSize::new(rect.width() as u32, rect.height() as u32),
      position: PhysicalPosition::new(rect.x(), rect.y()),
    }
  }
}
