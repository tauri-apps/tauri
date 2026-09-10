// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::{cell::Cell, os::raw::c_ulong, rc::Rc};
use tauri_runtime::ProgressBarState;
use tauri_runtime::dpi::PhysicalSize;
use tauri_utils::config::Color;
use winit::platform::gtk4::WindowExtGtk4;

use crate::window::AppWindow;

use super::{taskbar, utils::set_wm_state};

/// 24-bit X11 parent for CEF browser children.
///
/// GTK owns the toplevel layout and menu widgets, while CEF creates native X11
/// child windows. This host is kept sized to GTK's content box so CEF renders
/// below GTK UI instead of covering it. The host uses a 24-bit TrueColor visual
/// because CEF does not render correctly with the inherited GTK window visual on
/// all X11 setups.
///
/// Hierarchy:
/// - GtkApplicationWindow
///   - GtkBox
///     - menu
///     - content GtkBox
///   - CefX11Host, positioned over the content GtkBox
///     - CEF webview
///     - CEF webview
///     - CEF webview
pub(crate) struct CefX11Host {
  default_vbox: gtk::Box,
  xid: c_ulong,
  colormap: c_ulong,
  geometry: Rc<HostGeometry>,
}

/// Geometry of the X11 host, shared with the GTK `layout` handler that keeps it up to date.
#[derive(Default)]
struct HostGeometry {
  size: Cell<PhysicalSize<u32>>,
  /// Set when [`Self::size`] changed and the CEF children have not been laid out against it yet.
  needs_relayout: Cell<bool>,
}

impl CefX11Host {
  pub(crate) fn new(window: &dyn winit::window::Window) -> Option<Self> {
    use gtk::prelude::*;

    let gtk_window = window.gtk_window()?;
    let default_vbox = gtk::Box::new(gtk::Orientation::Vertical, 0);
    default_vbox.set_hexpand(true);
    default_vbox.set_vexpand(true);

    let webview_area = gtk::Box::new(gtk::Orientation::Vertical, 0);
    webview_area.set_hexpand(true);
    webview_area.set_vexpand(true);

    default_vbox.append(&webview_area);
    gtk_window.set_child(Some(&default_vbox));

    let parent_xid = window_xid(window);
    let initial_size = window.surface_size();
    let (xid, colormap) = create_cef_container(parent_xid, initial_size)?;

    let geometry = Rc::new(HostGeometry {
      size: Cell::new(initial_size),
      needs_relayout: Cell::new(false),
    });

    if let Some(surface) = gtk_window.surface() {
      let gtk_window = gtk_window.clone();
      let layout_webview_area = webview_area.clone();
      let layout_geometry = geometry.clone();
      surface.connect_layout(move |surface, _, _| {
        let size = set_cef_container_bounds(
          xid,
          &gtk_window,
          &layout_webview_area,
          surface.scale_factor().max(1) as f64,
        );
        if layout_geometry.size.replace(size) != size {
          // The content area changed without the toplevel being resized - a menu bar was
          // attached, hidden or shown - so winit emits no `SurfaceResized` and the CEF children
          // would keep the bounds computed against the previous host size.
          layout_geometry.needs_relayout.set(true);
        }
      });
    }

    Some(Self {
      default_vbox,
      xid,
      colormap,
      geometry,
    })
  }

  pub(crate) fn default_vbox(&self) -> gtk::Box {
    self.default_vbox.clone()
  }

  pub(crate) fn size(&self) -> PhysicalSize<u32> {
    self.geometry.size.get()
  }

  /// Whether the host was resized by GTK since the last time the CEF children were laid out.
  pub(crate) fn take_needs_relayout(&self) -> bool {
    self.geometry.needs_relayout.replace(false)
  }
}

impl Drop for CefX11Host {
  fn drop(&mut self) {
    super::utils::with_x11((), |xlib, display| unsafe {
      (xlib.XDestroyWindow)(display, self.xid);
      (xlib.XFreeColormap)(display, self.colormap);
    });
  }
}

impl AppWindow {
  pub(crate) fn cef_host_handle(&self) -> cef::sys::cef_window_handle_t {
    self.cef_host.xid as cef::sys::cef_window_handle_t
  }

  pub(crate) fn xid(&self) -> c_ulong {
    window_xid(self.window.as_ref())
  }

  pub(crate) fn raise_native(&self) {
    super::utils::activate_window(self.xid());
  }

  /// Applies the transient parent recorded by the window builder, if any.
  pub(crate) fn apply_transient_for(&self) {
    use gtk::prelude::GtkWindowExt;

    let Some(parent) = &self.attrs.transient_for else {
      return;
    };
    if let Some(window) = self.window.gtk_window() {
      window.set_transient_for(Some(parent));
    }
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    use gtk::prelude::*;

    if let Some(window) = self.window.gtk_window() {
      window.set_sensitive(enabled);
    }
  }

  pub(crate) fn is_enabled(&self) -> bool {
    use gtk::prelude::*;

    self
      .window
      .gtk_window()
      .map(|window| window.is_sensitive())
      .unwrap_or(true)
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    use gtk::prelude::*;

    const BACKGROUND_COLOR_CLASS: &str = "tauri-cef-window-background";

    let Some(window) = self.window.gtk_window() else {
      return;
    };

    let Some(color) = color else {
      window.remove_css_class(BACKGROUND_COLOR_CLASS);
      return;
    };

    let provider = gtk::CssProvider::new();
    let css = format!(
      ".{BACKGROUND_COLOR_CLASS} {{ background-color: rgba({}, {}, {}, {:.3}); }}",
      color.0,
      color.1,
      color.2,
      f64::from(color.3) / 255.
    );
    provider.load_from_bytes(&gtk::glib::Bytes::from_owned(css));
    gtk::style_context_add_provider_for_display(
      &gtk::prelude::WidgetExt::display(&window),
      &provider,
      gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
    window.add_css_class(BACKGROUND_COLOR_CLASS);
  }

  pub(crate) fn set_skip_taskbar(&self, skip: bool) {
    set_wm_state(self.xid(), skip, "_NET_WM_STATE_SKIP_TASKBAR", None);
  }

  pub(crate) fn set_visible_on_all_workspaces(&self, visible: bool) {
    set_wm_state(self.xid(), visible, "_NET_WM_STATE_STICKY", None);
  }

  pub(crate) fn set_progress_bar(&self, state: ProgressBarState) {
    taskbar::set_progress_bar(state);
  }
}

fn window_xid(window: &dyn winit::window::Window) -> c_ulong {
  let handle = window.window_handle().expect("failed to get window handle");
  match handle.as_raw() {
    RawWindowHandle::Xlib(handle) => handle.window as c_ulong,
    RawWindowHandle::Xcb(handle) => handle.window.get() as c_ulong,
    other => panic!("expected X11 window handle, got {other:?}"),
  }
}

fn create_cef_container(
  parent_xid: c_ulong,
  initial_size: PhysicalSize<u32>,
) -> Option<(c_ulong, c_ulong)> {
  use x11_dl::xlib::*;

  super::utils::with_x11(None, |xlib, display| unsafe {
    let screen = (xlib.XDefaultScreen)(display);
    let root = (xlib.XRootWindow)(display, screen);
    let mut visual_info: x11_dl::xlib::XVisualInfo = std::mem::zeroed();

    if (xlib.XMatchVisualInfo)(
      display,
      screen,
      24,
      x11_dl::xlib::TrueColor,
      &mut visual_info,
    ) == 0
    {
      return None;
    }

    let colormap = (xlib.XCreateColormap)(display, root, visual_info.visual, AllocNone);
    if colormap == 0 {
      return None;
    }

    let mut attrs: XSetWindowAttributes = std::mem::zeroed();
    attrs.event_mask = ExposureMask | StructureNotifyMask;
    attrs.colormap = colormap;
    attrs.border_pixel = 0;

    let xid = (xlib.XCreateWindow)(
      display,
      parent_xid as Window,
      0,
      0,
      initial_size.width,
      initial_size.height,
      0,
      visual_info.depth,
      InputOutput as _,
      visual_info.visual,
      CWEventMask | CWColormap | CWBorderPixel,
      &mut attrs,
    );
    if xid == 0 {
      (xlib.XFreeColormap)(display, colormap);
      return None;
    }

    (xlib.XMapWindow)(display, xid);
    Some((xid, colormap))
  })
}

/// Moves and resizes the X11 host over the GTK content area, returning its new size.
///
/// GTK4 widget geometry is in logical units while the X11 toplevel GDK creates is sized in device
/// pixels (logical * scale), so every value handed to `XMoveResizeWindow` must be scaled - without
/// it the host covers only 1/scale of the window on HiDPI screens.
fn set_cef_container_bounds(
  xid: c_ulong,
  gtk_window: &gtk::ApplicationWindow,
  webview_area: &gtk::Box,
  scale_factor: f64,
) -> PhysicalSize<u32> {
  use gtk::prelude::*;

  let width = (webview_area.width() as f64 * scale_factor).round() as u32;
  let height = (webview_area.height() as f64 * scale_factor).round() as u32;
  let point = gtk::graphene::Point::new(0.0, 0.0);
  let point = webview_area
    .compute_point(gtk_window, &point)
    .unwrap_or_else(|| gtk::graphene::Point::new(0.0, 0.0));
  let x = (point.x() as f64 * scale_factor).round() as i32;
  let y = (point.y() as f64 * scale_factor).round() as i32;

  super::utils::with_x11((), |xlib, display| unsafe {
    (xlib.XMoveResizeWindow)(display, xid as _, x, y, width, height);
  });

  PhysicalSize::new(width, height)
}
