// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use std::os::raw::c_ulong;
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
    let (xid, colormap) = create_cef_container(parent_xid, window.surface_size())?;

    if let Some(surface) = gtk_window.surface() {
      let gtk_window = gtk_window.clone();
      let layout_webview_area = webview_area.clone();
      surface.connect_layout(move |_, _, _| {
        set_cef_container_bounds(xid, &gtk_window, &layout_webview_area);
      });
    }

    Some(Self {
      default_vbox,
      xid,
      colormap,
    })
  }

  pub(crate) fn default_vbox(&self) -> gtk::Box {
    self.default_vbox.clone()
  }

  pub(crate) fn size(&self) -> PhysicalSize<u32> {
    super::utils::with_x11(PhysicalSize::default(), |xlib, display| unsafe {
      let mut root = 0;
      let mut x = 0;
      let mut y = 0;
      let mut width = 0;
      let mut height = 0;
      let mut border_width = 0;
      let mut depth = 0;

      (xlib.XGetGeometry)(
        display,
        self.xid as x11_dl::xlib::Window,
        &mut root,
        &mut x,
        &mut y,
        &mut width,
        &mut height,
        &mut border_width,
        &mut depth,
      );

      PhysicalSize::new(width, height)
    })
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

fn set_cef_container_bounds(
  xid: c_ulong,
  gtk_window: &gtk::ApplicationWindow,
  webview_area: &gtk::Box,
) {
  use gtk::prelude::*;

  let width = webview_area.width() as u32;
  let height = webview_area.height() as u32;
  let point = gtk::graphene::Point::new(0.0, 0.0);
  let point = webview_area
    .compute_point(gtk_window, &point)
    .unwrap_or_else(|| gtk::graphene::Point::new(0.0, 0.0));

  super::utils::with_x11((), |xlib, display| unsafe {
    (xlib.XMoveResizeWindow)(
      display,
      xid as _,
      point.x().round() as i32,
      point.y().round() as i32,
      width,
      height,
    );
  });
}
