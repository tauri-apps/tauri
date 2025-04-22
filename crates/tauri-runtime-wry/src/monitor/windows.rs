// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tao::dpi::{PhysicalPosition, PhysicalSize};
use tauri_runtime::dpi::PhysicalRect;

impl super::MonitorExt for tao::monitor::MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    use tao::platform::windows::MonitorHandleExtWindows;
    use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};
    let mut monitor_info = MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };
    let status = unsafe { GetMonitorInfoW(HMONITOR(self.hmonitor() as _), &mut monitor_info) };
    if status.as_bool() {
      PhysicalRect {
        size: PhysicalSize::new(
          (monitor_info.rcWork.right - monitor_info.rcWork.left) as u32,
          (monitor_info.rcWork.bottom - monitor_info.rcWork.top) as u32,
        ),
        position: PhysicalPosition::new(monitor_info.rcWork.left, monitor_info.rcWork.top),
      }
    } else {
      PhysicalRect {
        size: self.size(),
        position: self.position(),
      }
    }
  }
}
