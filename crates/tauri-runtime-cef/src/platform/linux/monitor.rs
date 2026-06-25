// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::os::raw::{c_int, c_long, c_uchar, c_ulong};
use tauri_runtime::dpi::{PhysicalPosition, PhysicalRect, PhysicalSize};
use winit::monitor::MonitorHandle;
use x11_dl::xlib;

use crate::platform::{MonitorExt, monitor_bounds};

use super::utils::{atom, with_x11};

impl MonitorExt for MonitorHandle {
  fn work_area(&self) -> PhysicalRect<i32, u32> {
    let bounds = monitor_bounds(self);
    x11_work_area(bounds).unwrap_or(bounds)
  }
}

fn x11_work_area(monitor_bounds: PhysicalRect<i32, u32>) -> Option<PhysicalRect<i32, u32>> {
  with_x11(None, |xlib, display| unsafe {
    let root = (xlib.XDefaultRootWindow)(display);
    let workareas = get_cardinal_property(xlib, display, root, "_NET_WORKAREA")?;

    let desktop = get_cardinal_property(xlib, display, root, "_NET_CURRENT_DESKTOP")
      .and_then(|desktops| desktops.first().copied())
      .and_then(|desktop| usize::try_from(desktop).ok())
      .unwrap_or(0);

    let offset = desktop.checked_mul(4)?;
    let workarea = workareas
      .get(offset..offset + 4)
      .or_else(|| workareas.get(0..4))?;

    let workarea = PhysicalRect {
      position: PhysicalPosition::new(
        i32::try_from(workarea[0]).ok()?,
        i32::try_from(workarea[1]).ok()?,
      ),
      size: PhysicalSize::new(
        u32::try_from(workarea[2]).ok()?,
        u32::try_from(workarea[3]).ok()?,
      ),
    };

    intersect_rects(monitor_bounds, workarea)
  })
}

fn get_cardinal_property(
  xlib: &xlib::Xlib,
  display: *mut xlib::Display,
  window: xlib::Window,
  name: &str,
) -> Option<Vec<c_long>> {
  let property = atom(xlib, display, name);
  if property == 0 {
    return None;
  }

  let mut actual_type: c_ulong = 0;
  let mut actual_format: c_int = 0;
  let mut nitems: c_ulong = 0;
  let mut bytes_after: c_ulong = 0;
  let mut data: *mut c_uchar = std::ptr::null_mut();

  let status = unsafe {
    (xlib.XGetWindowProperty)(
      display,
      window,
      property,
      0,
      256,
      0,
      xlib::XA_CARDINAL,
      &mut actual_type,
      &mut actual_format,
      &mut nitems,
      &mut bytes_after,
      &mut data,
    )
  };

  if status != xlib::Success as c_int || data.is_null() {
    return None;
  }

  let value = if actual_type == xlib::XA_CARDINAL && actual_format == 32 && bytes_after == 0 {
    let values = unsafe { std::slice::from_raw_parts(data.cast::<c_long>(), nitems as usize) };
    Some(values.to_vec())
  } else {
    None
  };

  unsafe {
    (xlib.XFree)(data.cast());
  }
  value
}

fn intersect_rects(
  a: PhysicalRect<i32, u32>,
  b: PhysicalRect<i32, u32>,
) -> Option<PhysicalRect<i32, u32>> {
  let left = a.position.x.max(b.position.x);
  let top = a.position.y.max(b.position.y);
  let right = rect_right(a).min(rect_right(b));
  let bottom = rect_bottom(a).min(rect_bottom(b));

  if right <= left || bottom <= top {
    return None;
  }

  Some(PhysicalRect {
    position: PhysicalPosition::new(left, top),
    size: PhysicalSize::new((right - left) as u32, (bottom - top) as u32),
  })
}

fn rect_right(rect: PhysicalRect<i32, u32>) -> i32 {
  rect
    .position
    .x
    .saturating_add(i32::try_from(rect.size.width).unwrap_or(i32::MAX))
}

fn rect_bottom(rect: PhysicalRect<i32, u32>) -> i32 {
  rect
    .position
    .y
    .saturating_add(i32::try_from(rect.size.height).unwrap_or(i32::MAX))
}
