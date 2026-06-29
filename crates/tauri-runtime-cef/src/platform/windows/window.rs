// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::{Icon, ProgressBarState, ProgressBarStatus};
use tauri_utils::config::Color;
use windows::Win32::UI::WindowsAndMessaging::{
  GWL_EXSTYLE, GetWindowLongW, SWP_FRAMECHANGED, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE,
  SWP_NOZORDER, SetWindowLongW, SetWindowPos, WS_EX_NOACTIVATE,
};
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

use crate::window::AppWindow;

use super::icon::icon_to_hicon;

impl AppWindow {
  pub(crate) fn hwnd(&self) -> HWND {
    let hwnd = self.raw_handle_as_cef_handle();
    HWND(hwnd.0 as _)
  }

  pub(crate) fn is_enabled(&self) -> bool {
    unsafe { IsWindowEnabled(self.hwnd()) }.as_bool()
  }

  pub(crate) fn set_enabled(&self, enabled: bool) {
    let _ = unsafe { EnableWindow(self.hwnd(), enabled) };
  }

  pub(crate) fn set_focusable(&self, focusable: bool) {
    let hwnd = self.hwnd();
    let mut style = unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) } as u32;
    if focusable {
      style &= !WS_EX_NOACTIVATE.0;
    } else {
      style |= WS_EX_NOACTIVATE.0;
    }

    unsafe {
      SetWindowLongW(hwnd, GWL_EXSTYLE, style as i32);
      let _ = SetWindowPos(
        hwnd,
        None,
        0,
        0,
        0,
        0,
        SWP_NOZORDER | SWP_NOMOVE | SWP_NOSIZE | SWP_FRAMECHANGED | SWP_NOACTIVATE,
      );
    }
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

  pub(crate) fn set_background_color(&self, _color: Option<Color>) {
    // TODO
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
