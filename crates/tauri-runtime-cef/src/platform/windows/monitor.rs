// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::dpi::{PhysicalPosition, PhysicalRect, PhysicalSize};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO};
use winit::monitor::MonitorHandle;

use crate::platform::{MonitorExt, monitor_bounds};

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let mut monitor_info = MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };

    let hmonitor = HMONITOR(self.native_id() as _);

    let status = unsafe { GetMonitorInfoW(hmonitor, &mut monitor_info) };
    if !status.as_bool() {
      return monitor_bounds(self);
    }

    let position = PhysicalPosition::new(monitor_info.rcWork.left, monitor_info.rcWork.top);
    let size = PhysicalSize::new(
      (monitor_info.rcWork.right - monitor_info.rcWork.left) as u32,
      (monitor_info.rcWork.bottom - monitor_info.rcWork.top) as u32,
    );
    PhysicalRect { position, size }
  }
}
