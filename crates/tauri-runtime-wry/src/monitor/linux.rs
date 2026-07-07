// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use gtk::gdk::prelude::MonitorExt;
use tao::platform::unix::MonitorHandleExtUnix;
use tauri_runtime::dpi::{LogicalPosition, LogicalSize, PhysicalRect};

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    // GTK4/GDK4 no longer exposes a portable work-area API, so Linux falls
    // back to full monitor geometry and does not account for panels/taskbars.
    let rect = self.gdk_monitor().geometry();
    let scale_factor = self.scale_factor();
    PhysicalRect {
      size: LogicalSize::new(rect.width() as u32, rect.height() as u32).to_physical(scale_factor),
      position: LogicalPosition::new(rect.x(), rect.y()).to_physical(scale_factor),
    }
  }
}
