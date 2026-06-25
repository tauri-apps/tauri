// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{cell::Cell, time::Instant};

use cef::application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol};
use objc2::{
  ClassType, DefinedClass, MainThreadMarker, define_class, extern_methods, msg_send, rc::Retained,
  runtime::Bool,
};
use objc2_app_kit::{
  NSApp, NSApplication, NSApplicationActivationOptions, NSEvent, NSRunningApplication,
};
use objc2_application_services::kProcessTransformToForegroundApplication;

use super::utils;

#[derive(Default)]
struct CefWinitApplicationIvars {
  handling_send_event: Cell<Bool>,
  last_dock_show: Cell<Option<Instant>>,
}

define_class!(
  #[unsafe(super(NSApplication))]
  #[ivars = CefWinitApplicationIvars]
  pub struct CefWinitApplication;

  impl CefWinitApplication {
    #[unsafe(method(sendEvent:))]
    unsafe fn send_event(&self, event: &NSEvent) {
      let was_handling = self.ivars().handling_send_event.get();
      self.ivars().handling_send_event.set(Bool::YES);
      let _: () = unsafe { msg_send![super(self), sendEvent: event] };
      self.ivars().handling_send_event.set(was_handling);
    }

    #[unsafe(method(tauriTransformProcessToForeground))]
    fn transform_process_to_foreground(&self) {
      utils::transform_process_type(kProcessTransformToForegroundApplication);
    }

    #[unsafe(method(tauriActivateCurrentApplication))]
    fn activate_current_application(&self) {
      let app = NSRunningApplication::currentApplication();
      app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
    }
  }

  unsafe impl CrAppControlProtocol for CefWinitApplication {
    #[unsafe(method(setHandlingSendEvent:))]
    unsafe fn set_handling_send_event(&self, handling_send_event: Bool) {
      self.ivars().handling_send_event.set(handling_send_event);
    }
  }

  unsafe impl CrAppProtocol for CefWinitApplication {
    #[unsafe(method(isHandlingSendEvent))]
    unsafe fn is_handling_send_event(&self) -> Bool {
      self.ivars().handling_send_event.get()
    }
  }

  unsafe impl CefAppProtocol for CefWinitApplication {}
);

impl CefWinitApplication {
  extern_methods! {
    #[unsafe(method(sharedApplication))]
    pub fn shared_application() -> Retained<Self>;
  }

  pub fn last_dock_show(&self) -> Option<Instant> {
    self.ivars().last_dock_show.get()
  }

  pub fn set_last_dock_show(&self, instant: Instant) {
    self.ivars().last_dock_show.set(Some(instant));
  }
}

pub fn setup_application() {
  let _ = CefWinitApplication::shared_application();
  let mtm = MainThreadMarker::new().expect("macOS application must start on the main thread");
  assert!(NSApp(mtm).isKindOfClass(CefWinitApplication::class()));
}
