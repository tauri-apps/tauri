// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  cell::Cell,
  time::{Duration, Instant},
};

use cef::application_mac::{CefAppProtocol, CrAppControlProtocol, CrAppProtocol};
use objc2::{
  ClassType, DefinedClass, MainThreadMarker, MainThreadOnly, define_class, extern_methods,
  msg_send,
  rc::Retained,
  runtime::{AnyObject, Bool, ProtocolObject},
};
use objc2_app_kit::{
  NSApp, NSApplication, NSApplicationActivationOptions, NSApplicationDelegate,
  NSApplicationTerminateReply, NSEvent, NSRunningApplication,
};
use objc2_application_services::kProcessTransformToForegroundApplication;
use objc2_foundation::{NSArray, NSObject, NSObjectProtocol, NSString, NSURL};

use super::utils;

#[derive(Default)]
pub(crate) struct CefWinitApplicationIvars {
  handling_send_event: Cell<Bool>,
  last_dock_show_ms: Cell<u64>,
  delegate: Cell<*const AppDelegate>,
}

pub(crate) enum AppDelegateEvent {
  TryTerminate,
  Reopen { has_visible_windows: bool },
  AccessibilityChanged { enabled: bool },
  OpenURLs { urls: Vec<url::Url> },
}

pub(crate) struct CefAppDelegateIvars {
  on_event: Box<dyn Fn(AppDelegateEvent)>,
}

define_class!(
  #[unsafe(super(NSObject))]
  #[name = "CefWinitAppDelegate"]
  #[ivars = CefAppDelegateIvars]
  #[thread_kind = MainThreadOnly]
  pub(crate) struct AppDelegate;

  unsafe impl NSObjectProtocol for AppDelegate {}

  #[allow(non_snake_case)]
  unsafe impl NSApplicationDelegate for AppDelegate {
    #[unsafe(method(application:openURLs:))]
    fn application_openURLs(&self, _application: &NSApplication, urls: &NSArray<NSURL>) {
      let urls = urls
        .iter()
        .filter_map(|ns_url| {
          ns_url
            .absoluteString()
            .and_then(|url_string| url_string.to_string().parse().ok())
        })
        .collect();

      self.emit(AppDelegateEvent::OpenURLs { urls });
    }

    #[unsafe(method(applicationShouldTerminate:))]
    fn applicationShouldTerminate(&self, _sender: &NSApplication) -> NSApplicationTerminateReply {
      NSApplicationTerminateReply::TerminateNow
    }

    #[unsafe(method(applicationShouldHandleReopen:hasVisibleWindows:))]
    fn applicationShouldHandleReopen_hasVisibleWindows(
      &self,
      _sender: &NSApplication,
      has_visible_windows: bool,
    ) -> bool {
      self.emit(AppDelegateEvent::Reopen {
        has_visible_windows,
      });
      false
    }

    #[unsafe(method(applicationSupportsSecureRestorableState:))]
    fn applicationSupportsSecureRestorableState(&self, _app: &NSApplication) -> bool {
      true
    }
  }

  impl AppDelegate {
    #[unsafe(method(tryToTerminateApplication:))]
    fn try_to_terminate_application(&self, _app: &NSApplication) {
      self.emit(AppDelegateEvent::TryTerminate);
    }

    #[unsafe(method(enableAccessibility:))]
    fn enable_accessibility(&self, enabled: Bool) {
      self.emit(AppDelegateEvent::AccessibilityChanged {
        enabled: enabled.as_bool(),
      });
    }
  }
);

define_class!(
  #[unsafe(super(NSApplication))]
  #[ivars = CefWinitApplicationIvars]
  pub(crate) struct CefWinitApplication;

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
      #[allow(deprecated)]
      app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
    }

    #[unsafe(method(terminate:))]
    unsafe fn terminate(&self, _sender: Option<&AnyObject>) {
      if let Some(delegate) = self.delegate() {
        delegate.emit(AppDelegateEvent::TryTerminate);
      }
    }

    #[unsafe(method(accessibilitySetValue:forAttribute:))]
    unsafe fn accessibility_set_value_for_attribute(
      &self,
      value: Option<&AnyObject>,
      attribute: Option<&NSString>,
    ) {
      if let (Some(value), Some(attribute)) = (value, attribute) {
        if attribute.to_string() == "AXEnhancedUserInterface" {
          let int_value: std::ffi::c_int = unsafe { msg_send![value, intValue] };
          if let Some(delegate) = self.delegate() {
            delegate.emit(AppDelegateEvent::AccessibilityChanged {
              enabled: int_value == 1,
            });
          }
        }
      }

      let _: () = unsafe {
        msg_send![super(self), accessibilitySetValue: value, forAttribute: attribute]
      };
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

impl AppDelegate {
  fn new(mtm: MainThreadMarker, on_event: Box<dyn Fn(AppDelegateEvent)>) -> Retained<Self> {
    let this = Self::alloc(mtm).set_ivars(CefAppDelegateIvars { on_event });
    unsafe { msg_send![super(this), init] }
  }

  fn emit(&self, event: AppDelegateEvent) {
    (self.ivars().on_event)(event);
  }
}

impl CefWinitApplication {
  extern_methods! {
    #[unsafe(method(sharedApplication))]
    pub fn shared_application() -> Retained<Self>;
  }

  pub fn last_dock_show(&self) -> Option<Instant> {
    match self.ivars().last_dock_show_ms.get() {
      0 => None,
      // Store elapsed milliseconds + 1 so the zero-initialized ivar can mean
      // "not set" for AppKit-created NSApplication instances.
      milliseconds => Some(utils::instant_epoch() + Duration::from_millis(milliseconds - 1)),
    }
  }

  pub fn set_last_dock_show(&self, instant: Instant) {
    let milliseconds = instant
      .saturating_duration_since(utils::instant_epoch())
      .as_millis()
      .try_into()
      .unwrap_or(u64::MAX);
    self
      .ivars()
      .last_dock_show_ms
      // Offset by one so `0` remains the zero-initialized "not set" sentinel.
      .set(milliseconds.saturating_add(1));
  }

  fn delegate(&self) -> Option<&AppDelegate> {
    unsafe { self.ivars().delegate.get().as_ref() }
  }
}

pub fn setup_application() {
  let _ = CefWinitApplication::shared_application();
  let mtm = MainThreadMarker::new().expect("macOS application must start on the main thread");
  assert!(NSApp(mtm).isKindOfClass(CefWinitApplication::class()));
}

pub(crate) fn set_application_event_handler(
  on_event: Box<dyn Fn(AppDelegateEvent)>,
) -> Retained<AppDelegate> {
  let mtm = MainThreadMarker::new().expect("macOS application must start on the main thread");
  let delegate = AppDelegate::new(mtm, on_event);
  let app = CefWinitApplication::shared_application();

  // `NSApplication.delegate` is weak. The runtime owns the retained delegate;
  // the app stores only a non-owning pointer because AppKit creates the
  // NSApplication singleton and its ivars must stay zero-initialized/no-drop.
  app.ivars().delegate.set(&*delegate as *const AppDelegate);
  app.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
  delegate
}
