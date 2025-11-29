use cef::{rc::*, *};

#[cfg(windows)]
pub use windows::BrowserDragHandlerData;

wrap_drag_handler! {
  pub struct BrowserDragHandler {
    data: BrowserDragHandlerData
  }

  impl DragHandler {
    fn on_draggable_regions_changed(
      &self,
      browser: Option<&mut Browser>,
      _frame: Option<&mut Frame>,
      regions: Option<&[cef::DraggableRegion]>
    ) {

      let Some(regions) = regions else { return; };
      let Some(browser) = browser else { return; };

      #[cfg(windows)]
      unsafe { windows::on_dragged_region_changed(&self.data, browser, regions) };
    }
  }
}

#[cfg(windows)]
pub mod windows {
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

  const K_PARENT_WND_PROC: PCWSTR = w!("CefParentWndProc");
  const K_DRAGGABLE_REGION: PCWSTR = w!("CefDraggableRegion");

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

    let enum_proc = if regions.is_empty() {
      let _ = RemovePropW(hwnd, K_DRAGGABLE_REGION);
      unsubclass_enum_windows_proc
    } else {
      let h_draggable_region = HANDLE(data.draggable_region.0 as _);
      let _ = SetPropW(hwnd, K_DRAGGABLE_REGION, Some(h_draggable_region));
      subclass_enum_windows_proc
    };

    unsafe {
      let _ = EnumChildWindows(
        Some(hwnd),
        Some(enum_proc),
        LPARAM(data.draggable_region.0 as _),
      );
    }
  }

  /// An enumerator proc to subclass windows.
  unsafe extern "system" fn subclass_enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
    subclass_window(hwnd, HRGN(lparam.0 as _));
    true.into()
  }

  /// An enumerator proc to un-subclass windows.
  unsafe extern "system" fn unsubclass_enum_windows_proc(hwnd: HWND, _lparam: LPARAM) -> BOOL {
    un_subclass_window(hwnd);
    true.into()
  }

  #[allow(non_snake_case)]
  pub fn LOWORD(dword: u32) -> u16 {
    (dword & 0xFFFF) as u16
  }

  #[allow(non_snake_case)]
  pub fn HIWORD(dword: u32) -> u16 {
    ((dword >> 16) & 0xFFFF) as u16
  }

  /// Subclasses a window to add draggable region support
  /// by intercepting the WM_NCHITTEST message and returning HTTRANSPARENT
  /// for points inside the draggable region so that the parent window can handle the dragging.
  fn subclass_window(hwnd: HWND, hrgn: HRGN) {
    let h_parent_wnd_proc = unsafe { GetPropW(hwnd, K_PARENT_WND_PROC) };
    if !h_parent_wnd_proc.is_invalid() {
      return;
    }

    // Reset last error
    unsafe { SetLastError(ERROR_SUCCESS) };

    let h_old_window_proc =
      unsafe { SetWindowLongPtrW(hwnd, GWLP_WNDPROC, subclassed_window_proc as isize) };
    if h_old_window_proc == 0 && unsafe { GetLastError() } != ERROR_SUCCESS {
      return;
    }

    unsafe {
      let _ = SetPropW(
        hwnd,
        K_PARENT_WND_PROC,
        Some(HANDLE(h_old_window_proc as _)),
      );
      let _ = SetPropW(hwnd, K_DRAGGABLE_REGION, Some(HANDLE(hrgn.0 as _)));
    }
  }

  /// Un-subclasses a window by restoring its original window procedure.
  fn un_subclass_window(hwnd: HWND) {
    let h_parent_window_proc = unsafe { GetPropW(hwnd, K_PARENT_WND_PROC) };
    if !h_parent_window_proc.is_invalid() {
      unsafe {
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, h_parent_window_proc.0 as isize);
      }
    }

    unsafe {
      let _ = RemovePropW(hwnd, K_DRAGGABLE_REGION);
      let _ = RemovePropW(hwnd, K_PARENT_WND_PROC);
    }
  }

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
    let h_parent_wnd_proc = unsafe { GetPropW(hwnd, K_PARENT_WND_PROC) };
    let draggable_region = {
      let hrgn = GetPropW(hwnd, K_DRAGGABLE_REGION);
      HRGN(hrgn.0 as _)
    };
    if msg == WM_NCHITTEST {
      let hit = CallWindowProcW(
        Some(std::mem::transmute::<
          *mut std::ffi::c_void,
          unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
        >(h_parent_wnd_proc.0)),
        hwnd,
        msg,
        wparam,
        lparam,
      );

      if hit.0 == HTCLIENT as isize {
        let points = POINTS {
          x: LOWORD(lparam.0 as u32) as i16,
          y: HIWORD(lparam.0 as u32) as i16,
        };
        let mut point = POINT {
          x: points.x as i32,
          y: points.y as i32,
        };

        let _ = ScreenToClient(hwnd, &mut point);

        if PtInRegion(draggable_region, point.x, point.y).as_bool() {
          return LRESULT(HTTRANSPARENT as isize);
        }
      }

      return hit;
    }

    CallWindowProcW(
      Some(std::mem::transmute::<
        *mut std::ffi::c_void,
        unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
      >(h_parent_wnd_proc.0)),
      hwnd,
      msg,
      wparam,
      lparam,
    )
  }

  unsafe extern "system" fn pfnsubclass(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
  ) -> LRESULT {
    let draggable_region = {
      let hrgn = GetPropW(hwnd, K_DRAGGABLE_REGION);
      HRGN(hrgn.0 as _)
    };

    let old_proc = GetPropW(hwnd, K_PARENT_WND_PROC);
    if old_proc.is_invalid() {
      return DefWindowProcW(hwnd, msg, wparam, lparam);
    }

    let old_proc = std::mem::transmute::<
      *mut std::ffi::c_void,
      unsafe extern "system" fn(HWND, u32, WPARAM, LPARAM) -> LRESULT,
    >(old_proc.0);

    if msg == WM_NCHITTEST {
      let hit = old_proc(hwnd, msg, wparam, lparam);

      if hit.0 == HTCLIENT as isize {
        let points = POINTS {
          x: LOWORD(lparam.0 as u32) as i16,
          y: HIWORD(lparam.0 as u32) as i16,
        };
        let mut point = POINT {
          x: points.x as i32,
          y: points.y as i32,
        };

        let _ = ScreenToClient(hwnd, &mut point);

        if PtInRegion(draggable_region, point.x, point.y).as_bool() {
          // If cursor is inside a draggable region return HTCAPTION to allow dragging.
          return LRESULT(HTCAPTION as _);
        }
      }
      return hit;
    }

    old_proc(hwnd, msg, wparam, lparam)
  }

  pub fn subclass_window_for_dragging(window: &mut cef::Window) {
    let hwnd = window.window_handle();
    let hwnd = HWND(hwnd.0 as _);

    let old_proc = unsafe { GetWindowLongPtrW(hwnd, GWLP_WNDPROC) };
    if old_proc == 0 {
      return;
    }

    unsafe {
      let _ = SetPropW(hwnd, K_PARENT_WND_PROC, Some(HANDLE(old_proc as _)));
      let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, pfnsubclass as isize);
    }
  }
}
