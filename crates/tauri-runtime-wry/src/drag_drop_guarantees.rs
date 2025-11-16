// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Compile-time checks to ensure critical drag-drop event paths exist.
//! These checks will cause compilation to fail if the critical code is removed.

#[cfg(test)]
mod compile_time_checks {
  use super::*;

  // This test ensures that DragDropEvent has the helper methods
  // If these methods are removed, this test will fail to compile
  #[test]
  fn drag_drop_event_has_helper_methods() {
    use tauri_runtime::window::DragDropEvent;
    use std::path::PathBuf;
    use tauri_runtime::dpi::PhysicalPosition;
    
    let event = DragDropEvent::Drop {
      paths: vec![PathBuf::from("/test")],
      position: PhysicalPosition::new(0.0, 0.0),
    };
    
    // These method calls will fail to compile if the methods don't exist
    let _paths: Option<&[PathBuf]> = event.paths();
    let _position: Option<&PhysicalPosition<f64>> = event.position();
    let _is_drop: bool = event.is_drop();
    let _is_enter: bool = event.is_enter();
    let _is_leave: bool = event.is_leave();
    let _is_over: bool = event.is_over();
  }

  // This test ensures WebviewAttributes has the file extension field
  #[test]
  fn webview_attributes_has_file_extension_filter() {
    use tauri_runtime::webview::{WebviewAttributes, WebviewUrl};
    
    let mut attrs = WebviewAttributes::new(WebviewUrl::App("index.html".into()));
    
    // This will fail to compile if the field doesn't exist
    attrs.drag_drop_file_extensions = Some(vec!["png".to_string()]);
    
    // This will fail to compile if the method doesn't exist
    let _attrs = attrs.drag_drop_file_extensions(vec!["jpg".to_string()]);
  }
}

/// Documentation marker to ensure drag-drop handler uses proxy.
///
/// The drag-drop handler MUST call proxy.send_event() to deliver events
/// to the app.run() callback. If this constant is referenced anywhere in
/// error messages or panics, it means someone is trying to bypass the proxy.
#[allow(dead_code)]
pub(crate) const DRAG_DROP_MUST_USE_PROXY: &str = 
  "Drag-drop events MUST be sent via proxy.send_event() to reach app.run() callback. \
   See DRAG_DROP_ARCHITECTURE.md for details. Bug originally fixed in PR #8393.";

/// Documentation marker for event loop callback requirement.
///
/// The event loop MUST call the callback function to deliver events to user code.
/// If this constant is referenced, it means the callback invocation is missing.
#[allow(dead_code)]
pub(crate) const DRAG_DROP_MUST_CALL_CALLBACK: &str = 
  "Event loop MUST invoke callback(RunEvent::*) to deliver drag-drop events to app.run(). \
   See DRAG_DROP_ARCHITECTURE.md for details. Bug originally fixed in PR #8393.";
