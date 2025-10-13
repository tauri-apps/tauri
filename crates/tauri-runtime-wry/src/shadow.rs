#![cfg(windows)]

use std::mem::size_of;
use windows::Win32::{
  Foundation::HWND,
  Graphics::Dwm::{
    DwmExtendFrameIntoClientArea, DwmSetWindowAttribute, DWMNCRP_DISABLED, DWMNCRP_ENABLED,
    DWMWA_BORDER_COLOR, DWMWA_NCRENDERING_POLICY, DWMWA_VISIBLE_FRAME_BORDER_THICKNESS,
    DWMWA_WINDOW_CORNER_PREFERENCE, DWMWCP_DONOTROUND,
  },
  UI::Controls::MARGINS,
};

const RESET_COLOR: u32 = u32::MAX;
const VISIBLE_BORDER_RESET: u32 = u32::MAX;
const DWMNCRP_USEWINDOWSTYLE: i32 = 0;

pub fn update(hwnd: isize, enabled: bool) {
  unsafe {
    let hwnd = HWND(hwnd as _);
    if enabled {
      enable(hwnd);
    } else {
      disable(hwnd);
    }
  }
}

pub fn reset(hwnd: isize) {
  unsafe {
    let hwnd = HWND(hwnd as _);
    let policy = DWMNCRP_USEWINDOWSTYLE;
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_NCRENDERING_POLICY,
      &policy as *const _ as _,
      size_of::<i32>() as u32,
    );
    let _ = DwmSetWindowAttribute(
      hwnd,
      DWMWA_VISIBLE_FRAME_BORDER_THICKNESS,
      &VISIBLE_BORDER_RESET as *const _ as _,
      size_of::<u32>() as u32,
    );
    let margins = MARGINS {
      cxLeftWidth: 0,
      cxRightWidth: 0,
      cyTopHeight: 0,
      cyBottomHeight: 0,
    };
    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
  }
}

unsafe fn enable(hwnd: HWND) {
  let policy = DWMNCRP_ENABLED.0;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_NCRENDERING_POLICY,
    &policy as *const _ as _,
    size_of::<i32>() as u32,
  );
  let corner = DWMWCP_DONOTROUND.0 as u32;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_WINDOW_CORNER_PREFERENCE,
    &corner as *const _ as _,
    size_of::<u32>() as u32,
  );
  let border_thickness: u32 = 0;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_VISIBLE_FRAME_BORDER_THICKNESS,
    &border_thickness as *const _ as _,
    size_of::<u32>() as u32,
  );
  let border_color: u32 = RESET_COLOR;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_BORDER_COLOR,
    &border_color as *const _ as _,
    size_of::<u32>() as u32,
  );
  let margins = MARGINS {
    cxLeftWidth: 1,
    cxRightWidth: 1,
    cyTopHeight: 0,
    cyBottomHeight: 1,
  };
  let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
}

unsafe fn disable(hwnd: HWND) {
  let policy = DWMNCRP_DISABLED.0;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_NCRENDERING_POLICY,
    &policy as *const _ as _,
    size_of::<i32>() as u32,
  );
  let border_color = RESET_COLOR;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_BORDER_COLOR,
    &border_color as *const _ as _,
    size_of::<u32>() as u32,
  );
  let reset_thickness: u32 = VISIBLE_BORDER_RESET;
  let _ = DwmSetWindowAttribute(
    hwnd,
    DWMWA_VISIBLE_FRAME_BORDER_THICKNESS,
    &reset_thickness as *const _ as _,
    size_of::<u32>() as u32,
  );
  let margins = MARGINS {
    cxLeftWidth: 0,
    cxRightWidth: 0,
    cyTopHeight: 0,
    cyBottomHeight: 0,
  };
  let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
}
