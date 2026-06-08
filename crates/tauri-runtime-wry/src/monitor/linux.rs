// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use gtk::prelude::MonitorExt;
use tao::platform::unix::MonitorHandleExtUnix;
use tauri_runtime::dpi::{LogicalPosition, LogicalSize, PhysicalRect};

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let rect = self.gdk_monitor().workarea();
    let scale_factor = self.scale_factor();
    PhysicalRect {
      size: LogicalSize::new(rect.width() as u32, rect.height() as u32).to_physical(scale_factor),
      position: LogicalPosition::new(rect.x(), rect.y()).to_physical(scale_factor),
    }
  }
}
