// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  platform::{EventLoopExt, MonitorExt},
  webview::AppWebview,
  window::AppWindow,
};
use cef::ImplBrowserHost;
use tauri_runtime::{
  Error, Icon, Result,
  dpi::{PhysicalPosition, PhysicalRect, PhysicalSize, Rect},
};
use tauri_utils::config::Color;
use windows::Win32::{
  Foundation::{HWND, POINT, RECT},
  Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO, MapWindowPoints},
  System::Com::{CLSCTX_SERVER, CoCreateInstance},
  UI::Shell::{ITaskbarList3, TaskbarList},
  UI::WindowsAndMessaging::{
    CreateIcon, DestroyIcon, GetCursorPos, GetParent, GetWindowRect, SW_HIDE, SW_SHOW,
    SWP_NOACTIVATE, SWP_NOZORDER, SetParent, SetWindowPos, ShowWindow,
  },
};
use winit::{event_loop::ActiveEventLoop, monitor::MonitorHandle};

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let mut monitor_info = MONITORINFO {
      cbSize: std::mem::size_of::<MONITORINFO>() as u32,
      ..Default::default()
    };

    let hmonitor = HMONITOR(self.native_id() as _);

    let status = unsafe { GetMonitorInfoW(hmonitor, &mut monitor_info) };
    if !status.as_bool() {
      return super::monitor_bounds(self);
    }

    let position = PhysicalPosition::new(monitor_info.rcWork.left, monitor_info.rcWork.top);
    let size = PhysicalSize::new(
      (monitor_info.rcWork.right - monitor_info.rcWork.left) as u32,
      (monitor_info.rcWork.bottom - monitor_info.rcWork.top) as u32,
    );
    PhysicalRect { position, size }
  }
}

impl AppWindow {
  pub(crate) fn hwnd(&self) -> HWND {
    let hwnd = self.raw_handle_as_cef_handle();
    HWND(hwnd.0 as _)
  }

  pub(crate) fn set_overlay_icon(&self, icon: Option<Icon<'static>>) {
    let Ok(taskbar) =
      (unsafe { CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_SERVER) })
    else {
      return;
    };

    let icon = icon.and_then(icon_to_hicon);
    let hwnd = self.hwnd();

    if let Some(icon) = icon {
      let _ = unsafe { taskbar.SetOverlayIcon(hwnd, icon, None) };
      let _ = unsafe { DestroyIcon(icon) };
    } else {
      let _ = unsafe { taskbar.SetOverlayIcon(hwnd, Default::default(), None) };
    }
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let _ = color;
    // TODO: implement native window background color on Windows.
  }
}

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_badge_count(&self, _count: Option<i64>, _desktop_filename: Option<String>) {
    // Unsupported on Windows
  }
  fn set_badge_label(&self, _label: Option<String>) {
    // Unsupported on Windows
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    let mut point = POINT::default();
    unsafe { GetCursorPos(&mut point) }.map_err(|_| Error::FailedToGetCursorPosition)?;
    Ok(PhysicalPosition::new(point.x as f64, point.y as f64))
  }
}

fn icon_to_hicon(icon: Icon<'static>) -> Option<windows::Win32::UI::WindowsAndMessaging::HICON> {
  let width = icon.width;
  let height = icon.height;
  let mut rgba = icon.rgba.into_owned();
  if width == 0 || height == 0 || rgba.len() != width as usize * height as usize * 4 {
    return None;
  }

  let mut and_mask = Vec::with_capacity(width as usize * height as usize);
  for pixel in rgba.chunks_exact_mut(4) {
    and_mask.push(pixel[3].wrapping_sub(u8::MAX));
    pixel.swap(0, 2);
  }

  unsafe {
    CreateIcon(
      None,
      width as i32,
      height as i32,
      1,
      32,
      and_mask.as_ptr(),
      rgba.as_ptr(),
    )
    .ok()
  }
}

impl AppWebview {
  pub(crate) fn hwnd(&self) -> HWND {
    let hwnd = self.host.window_handle();
    HWND(hwnd.0 as _)
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let _ = color;
    // Changing the native child HWND background only affects erase/fill, not
    // Chromium's rendered background. Creation still applies BrowserSettings.
  }

  pub(crate) fn bounds(&self) -> Option<Rect> {
    let hwnd = self.hwnd();

    let mut rect = RECT::default();
    unsafe {
      let parent = GetParent(hwnd).ok()?;
      if parent.0.is_null() {
        return None;
      }

      GetWindowRect(hwnd, &mut rect).ok()?;

      let mut points = [
        POINT {
          x: rect.left,
          y: rect.top,
        },
        POINT {
          x: rect.right,
          y: rect.bottom,
        },
      ];
      if MapWindowPoints(None, Some(parent), &mut points) == 0 {
        return None;
      }

      let x = points[0].x;
      let y = points[0].y;
      let width = (points[1].x - points[0].x).max(0) as u32;
      let height = (points[1].y - points[0].y).max(0) as u32;
      Some(Rect {
        position: PhysicalPosition::new(x, y).into(),
        size: PhysicalSize::new(width, height).into(),
      })
    }
  }

  pub(crate) fn reparent(&self, parent: &AppWindow) {
    let parent = parent.hwnd();
    let _ = unsafe { SetParent(self.hwnd(), Some(parent)) };
  }

  pub(crate) fn apply_visible(&self, visible: bool) {
    let _ = unsafe { ShowWindow(self.hwnd(), if visible { SW_SHOW } else { SW_HIDE }) };
  }

  pub(crate) fn apply_physical_bounds(&self, _scale: f64, x: i32, y: i32, width: i32, height: i32) {
    unsafe {
      let _ = SetWindowPos(
        self.hwnd(),
        None,
        x,
        y,
        width,
        height,
        SWP_NOZORDER | SWP_NOACTIVATE,
      );
    }
  }
}
