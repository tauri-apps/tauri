// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::PhysicalRect;
use gtk::prelude::MonitorExt;
use tao::{
  dpi::{PhysicalPosition, PhysicalSize},
  platform::unix::MonitorHandleExtUnix,
};

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect {
    let rect = self.gdk_monitor().workarea();
    PhysicalRect {
      size: PhysicalSize::new(rect.width() as u32, rect.height() as u32),
      position: PhysicalPosition::new(rect.x(), rect.y()),
    }
  }
}
