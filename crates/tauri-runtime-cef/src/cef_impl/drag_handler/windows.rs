use std::mem;

use cef::*;
use windows::core::{w, BOOL, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::Graphics::Gdi::*;
use windows::Win32::UI::WindowsAndMessaging::*;

#[derive(Clone)]
pub struct BrowserDragHandlerData {
  draggable_region: HRGN,
}

impl BrowserDragHandlerData {
  pub fn new() -> Self {
    Self {
      draggable_region: unsafe { CreateRectRgn(0, 0, 0, 0) },
    }
  }
}

const ORIGINAL_WND_PROP: PCWSTR = w!("CefOriginalWndProc");
const DRAGGABLE_REGION_PROP: PCWSTR = w!("CefDraggableRegion");

pub unsafe fn on_dragged_region_changed(
  data: &super::BrowserDragHandlerData,
  browser: &mut cef::Browser,
  regions: &[cef::DraggableRegion],
) {
  let Some(hwnd) = browser.host().map(|h| h.window_handle()) else {
    return;
  };
  let hwnd = HWND(hwnd.0 as _);

  // Reset the previous region
  let _ = unsafe { SetRectRgn(data.draggable_region, 0, 0, 0, 0) };

  // Combine all regions into a single region
  for region in regions {
    let hrgn = unsafe {
      CreateRectRgn(
        region.bounds.x,
        region.bounds.y,
        region.bounds.x + region.bounds.width,
        region.bounds.y + region.bounds.height,
      )
    };

    let mode = if region.draggable != 0 {
      RGN_OR
    } else {
      RGN_DIFF
    };

    unsafe {
      CombineRgn(
        Some(data.draggable_region),
        Some(data.draggable_region),
        Some(hrgn),
        mode,
      );
      let _ = DeleteObject(hrgn.into());
    }
  }

  // if there are regions, set the property for root window proc access
  // and subclass all child windows
  if !regions.is_empty() {
    let draggable_region = HANDLE(data.draggable_region.0 as _);
    let _ = SetPropW(hwnd, DRAGGABLE_REGION_PROP, Some(draggable_region));
    let _ = EnumChildWindows(
      Some(hwnd),
      Some(subclass_windows_proc),
      LPARAM(data.draggable_region.0 as _),
    );
  } else {
    // If no regions, remove the property from the root window and un-subclass all child windows
    let _ = RemovePropW(hwnd, DRAGGABLE_REGION_PROP);
    let _ = EnumChildWindows(
      Some(hwnd),
      Some(unsubclass_windows_proc),
      LPARAM(data.draggable_region.0 as _),
    );
  }
}

/// An enumerator proc to subclass windows.
unsafe extern "system" fn subclass_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
  subclass_window(hwnd, subclassed_window_proc, HRGN(lparam.0 as _));
  true.into()
}

/// An enumerator proc to un-subclass windows.
unsafe extern "system" fn unsubclass_windows_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
  un_subclass_window(hwnd);
  true.into()
}

/// Subclasses a window to add draggable region support
/// by intercepting the WM_NCHITTEST message and returning HTTRANSPARENT
/// for points inside the draggable region so that the parent window can handle the dragging
/// in [root_window_proc].
fn subclass_window(hwnd: HWND, proc: WindowProc, draggable_region: HRGN) {
  // If already subclassed, return early
  let orginial_wnd_proc = unsafe { GetPropW(hwnd, ORIGINAL_WND_PROP) };
  if !orginial_wnd_proc.is_invalid() {
    return;
  }

  // Reset last error
  unsafe { SetLastError(ERROR_SUCCESS) };

  // Set the new window procedure and get the orginal one
  let original_wnd_proc = unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, proc as isize) };
  if original_wnd_proc == 0 && unsafe { GetLastError() } != ERROR_SUCCESS {
    return;
  }

  unsafe {
    // Store the original window proc as a property for later use
    let _ = SetPropW(
      hwnd,
      ORIGINAL_WND_PROP,
      Some(HANDLE(original_wnd_proc as _)),
    );

    // Store the draggable region as a property for later use
    let _ = SetPropW(
      hwnd,
      DRAGGABLE_REGION_PROP,
      Some(HANDLE(draggable_region.0 as _)),
    );
  }
}

/// Un-subclasses a window by restoring its original window procedure.
fn un_subclass_window(hwnd: HWND) {
  let original_wnd_proc = unsafe { GetPropW(hwnd, ORIGINAL_WND_PROP) };
  if !original_wnd_proc.is_invalid() {
    unsafe {
      SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original_wnd_proc.0 as isize);
    }
  }

  // Remove the properties
  unsafe {
    let _ = RemovePropW(hwnd, DRAGGABLE_REGION_PROP);
    let _ = RemovePropW(hwnd, ORIGINAL_WND_PROP);
  }
}

/// Same as [WNDPROC] but without the Option wrapper.
type WindowProc = unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT;

/// The subclassed window procedure to handle WM_NCHITTEST messages
/// and return HTTRANSPARENT for points inside the draggable region.
///
/// This allows the parent window to handle dragging.
unsafe extern "system" fn subclassed_window_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let orginial_wnd_proc = unsafe { GetPropW(hwnd, ORIGINAL_WND_PROP) };
  let original_wnd_proc = mem::transmute::<_, WindowProc>(orginial_wnd_proc.0);

  let draggable_region = HRGN(GetPropW(hwnd, DRAGGABLE_REGION_PROP).0);

  if msg == WM_NCHITTEST {
    let hit = CallWindowProcW(Some(original_wnd_proc), hwnd, msg, wparam, lparam);

    // If the hit test is in the client area, check if it's in the draggable region
    if hit.0 == HTCLIENT as isize {
      let point = lparam_to_client_point(lparam, hwnd);

      // If the point is inside the draggable region, return HTTRANSPARENT
      // so the root window can handle the dragging
      if PtInRegion(draggable_region, point.x, point.y).as_bool() {
        return LRESULT(HTTRANSPARENT as isize);
      }
    }

    return hit;
  }

  // For other messages, call the original window procedure
  CallWindowProcW(Some(original_wnd_proc), hwnd, msg, wparam, lparam)
}

/// The root window procedure to handle WM_NCHITTEST messages
/// and return HTCAPTION for points inside the draggable region,
/// allowing the window to be dragged.
unsafe extern "system" fn root_window_proc(
  hwnd: HWND,
  msg: u32,
  wparam: WPARAM,
  lparam: LPARAM,
) -> LRESULT {
  let draggable_region = HRGN(GetPropW(hwnd, DRAGGABLE_REGION_PROP).0);

  let original_wnd_proc = GetPropW(hwnd, ORIGINAL_WND_PROP);

  // In the rare case where the original wnd proc is missing,
  // but somehow we got here, just call DefWindowProc
  if original_wnd_proc.is_invalid() {
    return DefWindowProcW(hwnd, msg, wparam, lparam);
  }
  let original_wnd_proc = std::mem::transmute::<_, WindowProc>(original_wnd_proc.0);

  if msg == WM_NCHITTEST {
    let hit = CallWindowProcW(Some(original_wnd_proc), hwnd, msg, wparam, lparam);

    // If the hit test is in the client area, check if it's in the draggable region
    if hit.0 == HTCLIENT as isize {
      let point = lparam_to_client_point(lparam, hwnd);

      // If the point is inside the draggable region, return HTCAPTION
      // to allow dragging the window
      if PtInRegion(draggable_region, point.x, point.y).as_bool() {
        return LRESULT(HTCAPTION as _);
      }
    }

    // For other areas, return the original hit test result
    return hit;
  }

  if msg == WM_NCLBUTTONDOWN {
    let point = lparam_to_client_point(lparam, hwnd);

    // If the point is inside the draggable region, call DefWindowProc
    // not the original wnd proc to ensure proper dragging behavior.
    if PtInRegion(draggable_region, point.x, point.y).as_bool() {
      return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }
  }

  // For other messages, call the original window procedure
  CallWindowProcW(Some(original_wnd_proc), hwnd, msg, wparam, lparam)
}

/// Subclasses the given window to handle draggable regions
/// by replacing its window procedure with `root_window_proc`
/// and storing the original procedure as a property to be called later.
pub fn subclass_window_for_dragging(window: &mut cef::Window) {
  let hwnd = window.window_handle();
  let hwnd = HWND(hwnd.0 as _);
  subclass_window(hwnd, root_window_proc, HRGN::default());
}

/// Converts a LPARAM from a mouse event to a POINT in client coordinates.
fn lparam_to_client_point(lparam: LPARAM, hwnd: HWND) -> POINT {
  let points = POINTS {
    x: LOWORD(lparam.0 as u32) as i16,
    y: HIWORD(lparam.0 as u32) as i16,
  };

  let mut point = POINT {
    x: points.x as i32,
    y: points.y as i32,
  };

  let _ = unsafe { ScreenToClient(hwnd, &mut point) };

  point
}

/// Extracts the low-order word from a 32-bit value.
#[allow(non_snake_case)]
pub fn LOWORD(dword: u32) -> u16 {
  (dword & 0xFFFF) as u16
}

/// Extracts the high-order word from a 32-bit value.
#[allow(non_snake_case)]
pub fn HIWORD(dword: u32) -> u16 {
  ((dword >> 16) & 0xFFFF) as u16
}
