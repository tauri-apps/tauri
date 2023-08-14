// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg(windows)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(clippy::upper_case_acronyms)]

use std::ffi::c_void;

use crate::utils::config::WindowEffectsConfig;
use crate::window::{Color, Effect};
use tauri_utils::platform::{get_function_impl, is_windows_7, windows_version};
use windows::Win32::Graphics::Dwm::{
  DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWINDOWATTRIBUTE,
};
use windows::Win32::{
  Foundation::{BOOL, HWND},
  Graphics::{
    Dwm::{DwmEnableBlurBehindWindow, DWM_BB_ENABLE, DWM_BLURBEHIND},
    Gdi::HRGN,
  },
};

pub fn apply_effects(window: HWND, effects: WindowEffectsConfig) {
  let WindowEffectsConfig { effects, color, .. } = effects;
  let effect = if let Some(effect) = effects.iter().find(|e| {
    matches!(
      e,
      Effect::Mica | Effect::MicaDark | Effect::MicaLight | Effect::Acrylic | Effect::Blur
    )
  }) {
    effect
  } else {
    return;
  };

  match effect {
    Effect::Blur => apply_blur(window, color),
    Effect::Acrylic => apply_acrylic(window, color),
    Effect::Mica => apply_mica(window, None),
    Effect::MicaDark => apply_mica(window, Some(true)),
    Effect::MicaLight => apply_mica(window, Some(false)),
    _ => unreachable!(),
  }
}

pub fn clear_effects(window: HWND) {
  clear_blur(window);
  clear_acrylic(window);
  clear_mica(window);
}

pub fn apply_blur(hwnd: HWND, color: Option<Color>) {
  if is_windows_7() {
    let bb = DWM_BLURBEHIND {
      dwFlags: DWM_BB_ENABLE,
      fEnable: true.into(),
      hRgnBlur: HRGN::default(),
      fTransitionOnMaximized: false.into(),
    };
    let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };
  } else if is_swca_supported() {
    unsafe { SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_ENABLE_BLURBEHIND, color) };
  } else {
    return;
  }
}

fn clear_blur(hwnd: HWND) {
  if is_windows_7() {
    let bb = DWM_BLURBEHIND {
      dwFlags: DWM_BB_ENABLE,
      fEnable: false.into(),
      hRgnBlur: HRGN::default(),
      fTransitionOnMaximized: false.into(),
    };
    let _ = unsafe { DwmEnableBlurBehindWindow(hwnd, &bb) };
  } else if is_swca_supported() {
    unsafe { SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None) };
  } else {
    return;
  }
}

pub fn apply_acrylic(hwnd: HWND, color: Option<Color>) {
  if is_backdroptype_supported() {
    unsafe {
      let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_TRANSIENTWINDOW as *const _ as _,
        4,
      );
    }
  } else if is_swca_supported() {
    unsafe {
      SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND, color);
    }
  } else {
    return;
  }
}

pub fn clear_acrylic(hwnd: HWND) {
  if is_backdroptype_supported() {
    unsafe {
      let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _,
        4,
      );
    }
  } else if is_swca_supported() {
    unsafe { SetWindowCompositionAttribute(hwnd, ACCENT_STATE::ACCENT_DISABLED, None) };
  } else {
    return;
  }
}

pub fn apply_mica(hwnd: HWND, dark: Option<bool>) {
  if let Some(dark) = dark {
    unsafe {
      DwmSetWindowAttribute(
        hwnd,
        DWMWA_USE_IMMERSIVE_DARK_MODE,
        &(dark as u32) as *const _ as _,
        4,
      );
    }
  }

  if is_backdroptype_supported() {
    unsafe {
      let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_MAINWINDOW as *const _ as _,
        4,
      );
    }
  } else if is_undocumented_mica_supported() {
    let _ = unsafe { DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &1 as *const _ as _, 4) };
  } else {
    return;
  }
}

pub fn clear_mica(hwnd: HWND) {
  if is_backdroptype_supported() {
    unsafe {
      let _ = DwmSetWindowAttribute(
        hwnd,
        DWMWA_SYSTEMBACKDROP_TYPE,
        &DWM_SYSTEMBACKDROP_TYPE::DWMSBT_DISABLE as *const _ as _,
        4,
      );
    }
  } else if is_undocumented_mica_supported() {
    let _ = unsafe { DwmSetWindowAttribute(hwnd, DWMWA_MICA_EFFECT, &0 as *const _ as _, 4) };
  } else {
    return;
  }
}

const DWMWA_MICA_EFFECT: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(1029i32);
const DWMWA_SYSTEMBACKDROP_TYPE: DWMWINDOWATTRIBUTE = DWMWINDOWATTRIBUTE(38i32);

#[repr(C)]
struct ACCENT_POLICY {
  AccentState: u32,
  AccentFlags: u32,
  GradientColor: u32,
  AnimationId: u32,
}

type WINDOWCOMPOSITIONATTRIB = u32;

#[repr(C)]
struct WINDOWCOMPOSITIONATTRIBDATA {
  Attrib: WINDOWCOMPOSITIONATTRIB,
  pvData: *mut c_void,
  cbData: usize,
}

#[derive(PartialEq)]
#[repr(C)]
enum ACCENT_STATE {
  ACCENT_DISABLED = 0,
  ACCENT_ENABLE_BLURBEHIND = 3,
  ACCENT_ENABLE_ACRYLICBLURBEHIND = 4,
}

macro_rules! get_function {
  ($lib:expr, $func:ident) => {
    get_function_impl(concat!($lib, '\0'), concat!(stringify!($func), '\0'))
      .map(|f| unsafe { std::mem::transmute::<windows::Win32::Foundation::FARPROC, $func>(f) })
  };
}

unsafe fn SetWindowCompositionAttribute(
  hwnd: HWND,
  accent_state: ACCENT_STATE,
  color: Option<Color>,
) {
  type SetWindowCompositionAttribute =
    unsafe extern "system" fn(HWND, *mut WINDOWCOMPOSITIONATTRIBDATA) -> BOOL;

  if let Some(set_window_composition_attribute) =
    get_function!("user32.dll", SetWindowCompositionAttribute)
  {
    let mut color = color.unwrap_or_default();

    let is_acrylic = accent_state == ACCENT_STATE::ACCENT_ENABLE_ACRYLICBLURBEHIND;
    if is_acrylic && color.3 == 0 {
      // acrylic doesn't like to have 0 alpha
      color.3 = 1;
    }

    let mut policy = ACCENT_POLICY {
      AccentState: accent_state as _,
      AccentFlags: if is_acrylic { 0 } else { 2 },
      GradientColor: (color.0 as u32)
        | (color.1 as u32) << 8
        | (color.2 as u32) << 16
        | (color.3 as u32) << 24,
      AnimationId: 0,
    };

    let mut data = WINDOWCOMPOSITIONATTRIBDATA {
      Attrib: 0x13,
      pvData: &mut policy as *mut _ as _,
      cbData: std::mem::size_of_val(&policy),
    };

    set_window_composition_attribute(hwnd, &mut data as *mut _ as _);
  }
}

#[allow(unused)]
#[repr(C)]
enum DWM_SYSTEMBACKDROP_TYPE {
  DWMSBT_DISABLE = 1,         // None
  DWMSBT_MAINWINDOW = 2,      // Mica
  DWMSBT_TRANSIENTWINDOW = 3, // Acrylic
  DWMSBT_TABBEDWINDOW = 4,    // Tabbed
}

fn is_swca_supported() -> bool {
  is_at_least_build(17763)
}

fn is_undocumented_mica_supported() -> bool {
  is_at_least_build(22000)
}

fn is_backdroptype_supported() -> bool {
  is_at_least_build(22523)
}

fn is_at_least_build(build: u32) -> bool {
  let v = windows_version().unwrap_or_default();
  v.2 >= build
}
