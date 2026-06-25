// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use tauri_runtime::ProgressBarState;
use tauri_utils::config::Color;

use crate::window::AppWindow;

use super::taskbar;

impl AppWindow {
  pub(crate) fn set_enabled(&self, enabled: bool) {
    let _ = (self, enabled);
    // TODO: implement native window enabled state on Linux/BSD.
  }

  pub(crate) fn is_enabled(&self) -> bool {
    let _ = self;
    // TODO: query native window enabled state on Linux/BSD.
    true
  }

  pub(crate) fn set_background_color(&self, color: Option<Color>) {
    let _ = (self, color);
    // TODO: implement native window background color on Linux/BSD.
  }

  pub(crate) fn set_progress_bar(&self, state: ProgressBarState) {
    taskbar::set_progress_bar(state);
  }
}
