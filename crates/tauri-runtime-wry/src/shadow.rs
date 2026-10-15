#![cfg(windows)]

//! DWM-based shadows for frameless windows.
//!
//! tao's `undecorated_with_shadows` emulation keeps the native `WS_CAPTION` /
//! `WS_THICKFRAME` styles alive and carves the client area back in through
//! `WM_NCCALCSIZE` insets. That leaves a visible frame band on the left,
//! right and bottom edges (the width of the resize frame), which is why
//! frameless windows cannot have borders and shadows independently
//! (tauri#13134). Instead, frameless windows stay plain (client area equal to
//! the window rect) and the shadow is anchored by extending the DWM frame
//! into the client area by one pixel on the left, right and bottom edges.
//!
//! - The top edge stays at zero: on Windows 10, any nonclient area at the top
//!   makes the system draw a full native titlebar.
//! - `DWMWA_BORDER_COLOR` is set to `COLOR_NONE` so the frame extension
//!   anchors the shadow without painting a visible outline. The attribute is
//!   Windows 11+; on Windows 10 the call fails silently and the system
//!   default outline remains.
//! - Corner rounding is intentionally left at the system default (rounded on
//!   Windows 11) instead of forcing `DWMWCP_DONOTROUND`, so frameless windows
//!   look like every other window on the platform.

use std::mem::size_of;
use windows::Win32::{
  Foundation::HWND,
  Graphics::Dwm::{
    DWMNCRP_ENABLED, DWMNCRP_USEWINDOWSTYLE, DWMWA_BORDER_COLOR, DWMWA_NCRENDERING_POLICY,
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute,
  },
  UI::Controls::MARGINS,
};

/// Restores the system default border outline (`DWMWA_COLOR_DEFAULT`).
const DEFAULT_BORDER_COLOR: u32 = u32::MAX;

/// Draws no border outline at all (`DWMWA_COLOR_NONE`, Windows 11+).
const NO_BORDER_COLOR: u32 = 0xFFFF_FFFE;

pub fn update(hwnd: isize, shadow: bool) {
  // The one pixel frame extension anchors the DWM shadow on the left, right
  // and bottom. The top stays at zero: on Windows 10, any nonclient area at
  // the top makes the system draw a full native titlebar.
  let (policy, border_color, margins) = if shadow {
    (
      DWMNCRP_ENABLED.0,
      NO_BORDER_COLOR,
      MARGINS {
        cxLeftWidth: 1,
        cyTopHeight: 0,
        cxRightWidth: 1,
        cyBottomHeight: 1,
      },
    )
  } else {
    // Restore the window-style driven DWM defaults. Used for decorated and
    // transparent windows as well as frameless windows without shadow.
    (
      DWMNCRP_USEWINDOWSTYLE.0,
      DEFAULT_BORDER_COLOR,
      MARGINS {
        cxLeftWidth: 0,
        cyTopHeight: 0,
        cxRightWidth: 0,
        cyBottomHeight: 0,
      },
    )
  };

  // SAFETY: `hwnd` is a live top-level window handle reconstructed from the
  // `isize` returned by `Window::hwnd()`, and `policy`, `border_color` and
  // `margins` are initialized values whose sizes match what the DWM APIs
  // expect for those attributes.
  unsafe {
    let hwnd = HWND(hwnd as _);
    if let Err(e) = DwmSetWindowAttribute(
      hwnd,
      DWMWA_NCRENDERING_POLICY,
      &policy as *const _ as _,
      size_of::<i32>() as u32,
    ) {
      log::warn!("failed to set the DWM nonclient rendering policy: {e}");
    }
    // Deliberately not logged: the attribute does not exist before Windows 11,
    // so a failure here is the expected path on older systems.
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_BORDER_COLOR,
      &border_color as *const _ as _,
      size_of::<u32>() as u32,
    );
    if let Err(e) = DwmExtendFrameIntoClientArea(hwnd, &margins) {
      log::warn!("failed to extend the DWM frame into the client area: {e}");
    }
  }
}
