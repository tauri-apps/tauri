use tauri_utils::config::WindowEffectsConfig;

use crate::{Runtime, Window};

#[cfg(target_os = "macos")]
mod macos;
#[cfg(windows)]
mod windows;

pub fn set_window_effects<R: Runtime>(
  window: &Window<R>,
  effects: Option<WindowEffectsConfig>,
) -> crate::Result<()> {
  if let Some(effects) = effects {
    #[cfg(windows)]
    {
      let hwnd = window.hwnd()?;
      windows::apply_effects(hwnd, effects);
    }
    #[cfg(target_os = "macos")]
    {
      let ns_window = window.ns_window()?;
      macos::apply_effects(ns_window, effects);
    }
  } else {
    #[cfg(windows)]
    {
      let hwnd = window.hwnd()?;
      windows::clear_effects(hwnd);
    }
  }
  Ok(())
}
