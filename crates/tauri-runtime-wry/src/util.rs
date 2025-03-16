// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg_attr(not(windows), allow(unused_imports))]
pub use imp::*;

#[cfg(not(windows))]
mod imp {}

#[cfg(windows)]
mod imp {
  use std::{iter::once, os::windows::ffi::OsStrExt};

  use once_cell::sync::Lazy;
  use windows::{
    core::{HRESULT, PCSTR, PCWSTR},
    Win32::{
      Foundation::*,
      Graphics::Gdi::*,
      System::LibraryLoader::{GetProcAddress, LoadLibraryW},
      UI::{HiDpi::*, WindowsAndMessaging::*},
    },
  };

  pub fn encode_wide(string: impl AsRef<std::ffi::OsStr>) -> Vec<u16> {
    string.as_ref().encode_wide().chain(once(0)).collect()
  }

  // Helper function to dynamically load function pointer.
  // `library` and `function` must be zero-terminated.
  pub(super) fn get_function_impl(library: &str, function: &str) -> FARPROC {
    let library = encode_wide(library);
    assert_eq!(function.chars().last(), Some('\0'));

    // Library names we will use are ASCII so we can use the A version to avoid string conversion.
    let module = unsafe { LoadLibraryW(PCWSTR::from_raw(library.as_ptr())) }.unwrap_or_default();
    if module.is_invalid() {
      return None;
    }

    unsafe { GetProcAddress(module, PCSTR::from_raw(function.as_ptr())) }
  }

  macro_rules! get_function {
    ($lib:expr, $func:ident) => {
      $crate::util::get_function_impl($lib, concat!(stringify!($func), '\0'))
        .map(|f| unsafe { std::mem::transmute::<_, $func>(f) })
    };
  }

  type GetDpiForWindow = unsafe extern "system" fn(hwnd: HWND) -> u32;
  type GetDpiForMonitor = unsafe extern "system" fn(
    hmonitor: HMONITOR,
    dpi_type: MONITOR_DPI_TYPE,
    dpi_x: *mut u32,
    dpi_y: *mut u32,
  ) -> HRESULT;
  type GetSystemMetricsForDpi =
    unsafe extern "system" fn(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32;

  static GET_DPI_FOR_WINDOW: Lazy<Option<GetDpiForWindow>> =
    Lazy::new(|| get_function!("user32.dll", GetDpiForWindow));
  static GET_DPI_FOR_MONITOR: Lazy<Option<GetDpiForMonitor>> =
    Lazy::new(|| get_function!("shcore.dll", GetDpiForMonitor));
  static GET_SYSTEM_METRICS_FOR_DPI: Lazy<Option<GetSystemMetricsForDpi>> =
    Lazy::new(|| get_function!("user32.dll", GetSystemMetricsForDpi));

  #[allow(non_snake_case)]
  pub unsafe fn hwnd_dpi(hwnd: HWND) -> u32 {
    let hdc = GetDC(Some(hwnd));
    if hdc.is_invalid() {
      return USER_DEFAULT_SCREEN_DPI;
    }
    if let Some(GetDpiForWindow) = *GET_DPI_FOR_WINDOW {
      // We are on Windows 10 Anniversary Update (1607) or later.
      match GetDpiForWindow(hwnd) {
        0 => USER_DEFAULT_SCREEN_DPI, // 0 is returned if hwnd is invalid
        dpi => dpi,
      }
    } else if let Some(GetDpiForMonitor) = *GET_DPI_FOR_MONITOR {
      // We are on Windows 8.1 or later.
      let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
      if monitor.is_invalid() {
        return USER_DEFAULT_SCREEN_DPI;
      }

      let mut dpi_x = 0;
      let mut dpi_y = 0;
      if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &mut dpi_x, &mut dpi_y).is_ok() {
        dpi_x
      } else {
        USER_DEFAULT_SCREEN_DPI
      }
    } else {
      // We are on Vista or later.
      if IsProcessDPIAware().as_bool() {
        // If the process is DPI aware, then scaling must be handled by the application using
        // this DPI value.
        GetDeviceCaps(Some(hdc), LOGPIXELSX) as u32
      } else {
        // If the process is DPI unaware, then scaling is performed by the OS; we thus return
        // 96 (scale factor 1.0) to prevent the window from being re-scaled by both the
        // application and the WM.
        USER_DEFAULT_SCREEN_DPI
      }
    }
  }

  #[allow(non_snake_case)]
  pub unsafe fn get_system_metrics_for_dpi(nindex: SYSTEM_METRICS_INDEX, dpi: u32) -> i32 {
    if let Some(GetSystemMetricsForDpi) = *GET_SYSTEM_METRICS_FOR_DPI {
      GetSystemMetricsForDpi(nindex, dpi)
    } else {
      GetSystemMetrics(nindex)
    }
  }
}
