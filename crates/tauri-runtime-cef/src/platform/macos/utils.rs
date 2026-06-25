use objc2::rc::Retained;
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
