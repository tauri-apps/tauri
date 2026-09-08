// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{ffi::c_void, sync::OnceLock, time::Instant};

use objc2::{
  ClassType,
  ffi::{OBJC_ASSOCIATION_RETAIN_NONATOMIC, objc_getAssociatedObject, objc_setAssociatedObject},
  rc::Retained,
  runtime::AnyObject,
};
use objc2_app_kit::NSColor;
use objc2_application_services::{
  ProcessApplicationTransformState, TransformProcessType, kCurrentProcess,
};
use tauri_utils::config::Color;

#[repr(C)]
#[allow(non_snake_case)]
struct ProcessSerialNumber {
  highLongOfPSN: u32,
  lowLongOfPSN: u32,
}

pub fn transform_process_type(transform_state: ProcessApplicationTransformState) {
  let process_serial_number = ProcessSerialNumber {
    highLongOfPSN: 0,
    lowLongOfPSN: kCurrentProcess,
  };

  unsafe {
    let serial = (&process_serial_number as *const ProcessSerialNumber).cast();
    let _ = TransformProcessType(serial, transform_state);
  }
}

pub fn ns_color_from_tauri_color(color: Color) -> Retained<NSColor> {
  let Color(red, green, blue, alpha) = color;
  let scale = u8::MAX as f64;
  NSColor::colorWithSRGBRed_green_blue_alpha(
    red as f64 / scale,
    green as f64 / scale,
    blue as f64 / scale,
    alpha as f64 / scale,
  )
}

pub fn instant_epoch() -> Instant {
  static INSTANT_EPOCH: OnceLock<Instant> = OnceLock::new();
  *INSTANT_EPOCH.get_or_init(Instant::now)
}

pub(crate) fn set_associated_object<O: ClassType>(
  object: &O,
  key: *const c_void,
  value: &AnyObject,
) {
  unsafe {
    objc_setAssociatedObject(
      object as *const O as *mut AnyObject,
      key,
      value as *const AnyObject as *mut AnyObject,
      OBJC_ASSOCIATION_RETAIN_NONATOMIC,
    );
  }
}

pub(crate) fn associated_object<O: ClassType>(
  object: &O,
  key: *const c_void,
) -> Option<Retained<AnyObject>> {
  let value = unsafe { objc_getAssociatedObject(object as *const O as *const AnyObject, key) };
  // SAFETY: the association is only ever written by `set_associated_object`,
  // which stores a valid object pointer under this key.
  unsafe { Retained::retain(value.cast_mut()) }
}
