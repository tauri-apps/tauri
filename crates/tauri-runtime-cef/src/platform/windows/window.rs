// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::num::NonZeroU32;

use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use tauri_runtime::{Icon, ProgressBarState, ProgressBarStatus};
use tauri_utils::config::Color;
use windows::Win32::{
  Foundation::{HWND, RECT},
  Graphics::Dwm::{DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute},
  System::Com::{CLSCTX_SERVER, CoCreateInstance},
  UI::{
    Input::KeyboardAndMouse::{EnableWindow, IsWindowEnabled},
    Shell::{
      ITaskbarList3, TBPF_ERROR, TBPF_INDETERMINATE, TBPF_NOPROGRESS, TBPF_NORMAL, TBPF_PAUSED,
      TaskbarList,
    },
    WindowsAndMessaging::DestroyIcon,
  },
};

use crate::{window::AppWindow, window_handle::SoftbufferWindowHandle};

use super::icon::icon_to_hicon;

impl AppWindow {
  pub(crate) fn raw_cef_handle(&self) -> cef::sys::cef_window_handle_t {
    cef::sys::HWND(self.hwnd().0 as *mut _)
  }

  pub(crate) fn hwnd(&self) -> HWND {
    let handle = self
      .window
      .window_handle()
      .expect("failed to get window handle");
    match handle.as_raw() {
      RawWindowHandle::Win32(handle) => HWND(handle.hwnd.get() as _),
      other => panic!("expected Win32 window handle, got {other:?}"),
    }
  }

  pub(crate) fn is_enabled(&self) -> bool {
    unsafe { IsWindowEnabled(self.hwnd()) }.as_bool()
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    let _ = unsafe { EnableWindow(self.hwnd(), enabled) };
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

  pub(crate) fn set_progress_bar(&self, state: ProgressBarState) {
    let Ok(taskbar) =
      (unsafe { CoCreateInstance::<_, ITaskbarList3>(&TaskbarList, None, CLSCTX_SERVER) })
    else {
      return;
    };

    let hwnd = self.hwnd();
    if let Some(status) = state.status {
      let flag = match status {
        ProgressBarStatus::None => TBPF_NOPROGRESS,
        ProgressBarStatus::Normal => TBPF_NORMAL,
        ProgressBarStatus::Indeterminate => TBPF_INDETERMINATE,
        ProgressBarStatus::Paused => TBPF_PAUSED,
        ProgressBarStatus::Error => TBPF_ERROR,
      };
      let _ = unsafe { taskbar.SetProgressState(hwnd, flag) };
    }

    if let Some(progress) = state.progress {
      let _ = unsafe { taskbar.SetProgressValue(hwnd, progress.min(100), 100) };
    }
  }

  pub(crate) fn set_background_color(&mut self, _color: Option<Color>) {
    // Nothing to do here, the background color is already updated in the window attributes,
    // and the background surface will be drawn in the next frame.
    // Just request a redraw.
    self.window.request_redraw();
  }

  /// Paints the window's own background over everything its webviews do not
  /// cover — the window is `WS_CLIPCHILDREN`, so live webviews clip themselves
  /// out of this paint.
  ///
  /// This is the only thing that ever paints the window itself: winit registers
  /// its window class without a background brush, and Windows leaves the pixels
  /// a destroyed child window drew last sitting in the parent's client area. A
  /// closed webview would otherwise keep showing its final frame — visible, and
  /// backed by no window at all — for as long as the window lived. Destroying
  /// the webview's window invalidates the area it covered, so painting on every
  /// redraw is what clears it.
  pub(crate) fn draw_background_surface(&mut self) {
    let size = self.window.surface_size();
    let (Some(width), Some(height)) = (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
    else {
      return;
    };

    if self.background_surface.is_none() {
      let Some(handle) = SoftbufferWindowHandle::new(self.window.as_ref()) else {
        return;
      };
      let Ok(context) = softbuffer::Context::new(handle) else {
        return;
      };
      let Ok(surface) = softbuffer::Surface::new(&context, handle) else {
        return;
      };
      self.background_surface = Some(surface);
    }

    let Some(surface) = &mut self.background_surface else {
      return;
    };

    let color = match self.attrs.background_color {
      Some(Color(r, g, b, _)) => (b as u32) | ((g as u32) << 8) | ((r as u32) << 16),
      // A transparent window paints nothing so the desktop shows through, while
      // an ordinary one falls back to the opaque white a blank browser shows.
      None if self.attrs.inner.transparent => 0,
      None => 0x00ff_ffff,
    };

    if surface.resize(width, height).is_ok()
      && let Ok(mut buffer) = surface.buffer_mut()
    {
      buffer.fill(color);
      let _ = buffer.present();
    }
  }

  /// The visible frame height reported by DWM (`DWMWA_EXTENDED_FRAME_BOUNDS`).
  ///
  /// winit's `outer_size` includes the invisible resize/shadow border, which
  /// throws off vertical centering for decorated windows. The DWM extended
  /// frame bounds describe the actually-visible window rectangle, so its height
  /// is what should be used when centering. Returns `None` on failure.
  pub(crate) fn dwm_visible_frame_height(&self) -> Option<u32> {
    let mut rect = RECT::default();
    let result = unsafe {
      DwmGetWindowAttribute(
        self.hwnd(),
        DWMWA_EXTENDED_FRAME_BOUNDS,
        &mut rect as *mut _ as *mut _,
        std::mem::size_of::<RECT>() as u32,
      )
    };
    result.ok()?;
    Some((rect.bottom - rect.top) as u32)
  }
}
