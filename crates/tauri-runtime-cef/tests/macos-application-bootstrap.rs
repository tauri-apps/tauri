// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// AppKit requires the process main thread, which the standard test harness does not use.
#[cfg(target_os = "macos")]
fn main() {
  use objc2::{ClassType, MainThreadMarker, msg_send, runtime::Bool};
  use objc2_app_kit::NSApplication;

  let mtm = MainThreadMarker::new().expect("test must run on the main thread");
  if std::env::args().nth(1).as_deref() == Some("--existing-application") {
    let _ = NSApplication::sharedApplication(mtm);
    assert!(std::panic::catch_unwind(tauri_runtime_cef::prepare_macos_application).is_err());
    return;
  }
  assert!(
    std::process::Command::new(std::env::current_exe().unwrap())
      .arg("--existing-application")
      .status()
      .unwrap()
      .success()
  );

  assert!(
    std::thread::spawn(tauri_runtime_cef::prepare_macos_application)
      .join()
      .is_err()
  );

  tauri_runtime_cef::prepare_macos_application();
  let application = NSApplication::sharedApplication(mtm);
  assert!(!std::ptr::eq(application.class(), NSApplication::class()));

  // Native startup UI can now request the singleton without replacing CEF's application.
  let native_application = NSApplication::sharedApplication(mtm);
  assert!(std::ptr::eq(&*application, &*native_application));
  tauri_runtime_cef::prepare_macos_application();
  assert!(std::ptr::eq(
    &*application,
    &*NSApplication::sharedApplication(mtm)
  ));

  // CEF's event scopes use these selectors even before an event-loop delegate exists.
  unsafe {
    let handling: Bool = msg_send![&*application, isHandlingSendEvent];
    assert!(!handling.as_bool());
    let _: () = msg_send![&*application, setHandlingSendEvent: Bool::YES];
    let handling: Bool = msg_send![&*application, isHandlingSendEvent];
    assert!(handling.as_bool());
    let _: () = msg_send![&*application, setHandlingSendEvent: Bool::NO];
  }
}

#[cfg(not(target_os = "macos"))]
fn main() {}
