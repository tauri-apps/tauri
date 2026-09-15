// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "ohos")]
use tao::window::Window;

impl super::WindowExt for Window {
  fn set_enabled(&self, _enabled: bool) {
    // OHOS/HarmonyOS window enabling not yet implemented
  }

  fn is_enabled(&self) -> bool {
    true
  }

  fn center(&self) {
    // OHOS/HarmonyOS window centering not yet implemented
  }
}
