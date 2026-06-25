// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::{Icon, ProgressBarState, ProgressBarStatus};
use tauri_utils::config::Color;
use windows::Win32::{
  Foundation::HWND,
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
}
