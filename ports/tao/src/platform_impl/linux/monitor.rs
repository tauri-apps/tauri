// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use std::collections::VecDeque;

use gtk4::{
  gdk::{
    self,
    prelude::{DisplayExt, MonitorExt},
    Monitor,
  },
  gio::prelude::{ListModelExt, ListModelExtManual},
  glib::object::Cast,
  prelude::{NativeExt, WidgetExt},
};

use crate::{
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize},
  monitor::{MonitorHandle as RootMonitorHandle, VideoMode as RootVideoMode},
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonitorHandle {
  pub(crate) monitor: gdk::Monitor,
}

impl MonitorHandle {
  pub fn new(monitor: gdk::Monitor) -> Self {
    Self { monitor }
  }

  #[inline]
  pub fn name(&self) -> Option<String> {
    self.monitor.model().map(|s| s.as_str().to_string())
  }

  #[inline]
  pub fn size(&self) -> PhysicalSize<u32> {
    let rect = self.monitor.geometry();
    LogicalSize {
      width: rect.width() as u32,
      height: rect.height() as u32,
    }
    .to_physical(self.scale_factor())
  }

  #[inline]
  pub fn position(&self) -> PhysicalPosition<i32> {
    let rect = self.monitor.geometry();
    LogicalPosition {
      x: rect.x(),
      y: rect.y(),
    }
    .to_physical(self.scale_factor())
  }

  #[inline]
  pub fn scale_factor(&self) -> f64 {
    self.monitor.scale_factor() as f64
  }

  #[inline]
  pub fn video_modes(&self) -> Box<dyn Iterator<Item = RootVideoMode>> {
    Box::new(Vec::new().into_iter())
  }
}

unsafe impl Send for MonitorHandle {}
unsafe impl Sync for MonitorHandle {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct VideoMode;

impl VideoMode {
  #[inline]
  pub fn size(&self) -> PhysicalSize<u32> {
    panic!("VideoMode is unsupported on Linux.")
  }

  #[inline]
  pub fn bit_depth(&self) -> u16 {
    panic!("VideoMode is unsupported on Linux.")
  }

  #[inline]
  pub fn refresh_rate(&self) -> u16 {
    panic!("VideoMode is unsupported on Linux.")
  }

  #[inline]
  pub fn monitor(&self) -> RootMonitorHandle {
    panic!("VideoMode is unsupported on Linux.")
  }
}

#[inline]
pub fn current_monitor<W: WidgetExt + NativeExt>(window: &W) -> Option<RootMonitorHandle> {
  // `.surface()` returns `None` if the window is invisible;
  // we fallback to the primary monitor
  window
    .surface()
    .and_then(|surface| window.display().monitor_at_surface(&surface))
    .or_else(|| first_monitor(&window.display()))
    .map(|monitor| {
      let handle = MonitorHandle { monitor };
      RootMonitorHandle { inner: handle }
    })
}

#[inline]
fn first_monitor<W: DisplayExt>(display: &W) -> Option<Monitor> {
  display
    .monitors()
    .item(0)
    .and_then(|m| m.dynamic_cast::<Monitor>().ok())
}

#[inline]
pub fn primary_monitor<W: DisplayExt>(display: &W) -> Option<RootMonitorHandle> {
  first_monitor(display).map(|monitor| RootMonitorHandle {
    inner: MonitorHandle::new(monitor),
  })
}

#[inline]
pub fn available_monitors<W: DisplayExt>(display: &W) -> VecDeque<MonitorHandle> {
  display
    .monitors()
    .iter::<Monitor>()
    .map(|monitor| MonitorHandle::new(monitor.unwrap()))
    .collect()
}
