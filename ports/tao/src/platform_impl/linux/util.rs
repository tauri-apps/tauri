// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  dpi::{LogicalPosition, LogicalSize, PhysicalPosition},
  error::ExternalError,
  window::WindowSizeConstraints,
};
use gtk4::{
  gdk,
  glib::{
    self,
    object::{Cast, IsA},
    SignalHandlerId,
  },
  prelude::{DisplayExt, GtkWindowExt, NativeExt, SeatExt, SurfaceExt, WidgetExt},
};
use std::{cell::RefCell, rc::Rc};

#[inline]
pub fn on_window_realized<W, F>(window: &W, f: F) -> SignalHandlerId
where
  W: IsA<gtk4::Widget>,
  F: Fn(&W) + 'static,
{
  // If the window is already realized, we won't get the signal initially.
  if window.is_realized() {
    f(window)
  }
  window.connect_realize(f)
}

#[inline]
pub fn surface_as_toplevel(surface: gdk::Surface) -> Result<gdk::Toplevel, gdk::Surface> {
  surface.clone().downcast::<gdk::Toplevel>()
}

#[inline]
pub fn default_pointer(display: &gdk::Display) -> Option<gdk::Device> {
  display
    .default_seat()
    .and_then(|seat: gtk4::gdk::Seat| seat.pointer())
}

#[inline]
pub fn cursor_position<W: GtkWindowExt + WidgetExt + NativeExt>(
  window: &W,
) -> Result<PhysicalPosition<f64>, ExternalError> {
  default_pointer(&window.display())
    .and_then(|pointer| {
      window
        .surface()
        .and_then(|surface| surface.device_position(&pointer))
        .map(|(x, y, _)| LogicalPosition::new(x, y).to_physical::<f64>(window.scale_factor() as _))
    })
    .ok_or(ExternalError::Os(os_error!(super::OsError)))
}

pub fn set_size_constraints<W: GtkWindowExt + WidgetExt>(
  window: &W,
  constraints: WindowSizeConstraints,
) {
  let scale_factor = window.scale_factor() as f64;

  let min_size: LogicalSize<i32> = constraints.min_size_logical(scale_factor);

  let w = if min_size.width > -1 {
    min_size.width
  } else {
    -1
  };
  let h = if min_size.height > -1 {
    min_size.height
  } else {
    -1
  };

  window.set_size_request(w, h);
}

pub struct WindowMaximizeProcess<W: GtkWindowExt + WidgetExt> {
  window: W,
  resizable: bool,
  step: u8,
}

impl<W: GtkWindowExt + WidgetExt> WindowMaximizeProcess<W> {
  pub fn new(window: W, resizable: bool) -> Rc<RefCell<Self>> {
    Rc::new(RefCell::new(Self {
      window,
      resizable,
      step: 0,
    }))
  }

  pub fn next_step(&mut self) -> glib::ControlFlow {
    match self.step {
      0 => {
        self.window.set_resizable(true);
        self.step += 1;
        glib::ControlFlow::Continue
      }
      1 => {
        self.window.maximize();
        self.step += 1;
        glib::ControlFlow::Continue
      }
      2 => {
        self.window.set_resizable(self.resizable);
        glib::ControlFlow::Break
      }
      _ => glib::ControlFlow::Break,
    }
  }
}
