// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc2::MainThreadMarker;
use objc2_app_kit::{NSApp, NSApplication, NSApplicationActivationPolicy, NSEvent, NSScreen};
use objc2_foundation::{NSPoint, NSString};
use tauri_runtime::{Error, ProgressBarState, Result, dpi::PhysicalPosition};
use winit::event_loop::ActiveEventLoop;

use crate::platform::EventLoopExt;

use super::{application::CefWinitApplication, progress};

impl EventLoopExt for dyn ActiveEventLoop + '_ {
  fn set_activation_policy(&self, policy: tauri_runtime::ActivationPolicy) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let policy = match policy {
      tauri_runtime::ActivationPolicy::Regular => NSApplicationActivationPolicy::Regular,
      tauri_runtime::ActivationPolicy::Accessory => NSApplicationActivationPolicy::Accessory,
      tauri_runtime::ActivationPolicy::Prohibited => NSApplicationActivationPolicy::Prohibited,
      _ => NSApplicationActivationPolicy::Regular,
    };
    app.setActivationPolicy(policy);
  }

  fn set_dock_visibility(&self, visible: bool) {
    let Some(_mtm) = MainThreadMarker::new() else {
      return;
    };

    let app = CefWinitApplication::shared_application();
    app.set_dock_visibility(visible);
  }

  fn show_application(&self) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };

    NSApp(mtm).unhide(None);
  }

  fn hide_application(&self) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };

    NSApp(mtm).hide(None);
  }

  fn set_progress_bar(&self, state: ProgressBarState) {
    progress::set_dock_progress_bar(state);
  }

  fn set_badge_count(&self, count: Option<i64>, _desktop_filename: Option<String>) {
    self.set_badge_label(count.map(|count| count.to_string()));
  }

  fn set_badge_label(&self, label: Option<String>) {
    let Some(mtm) = MainThreadMarker::new() else {
      return;
    };

    let app = NSApplication::sharedApplication(mtm);
    let dock_tile = app.dockTile();
    let ns_label = label.map(|label| NSString::from_str(&label));
    dock_tile.setBadgeLabel(ns_label.as_deref());
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    let Some(mtm) = MainThreadMarker::new() else {
      return Err(Error::FailedToGetCursorPosition);
    };

    // `NSEvent::mouseLocation` uses a bottom-left origin; flip to top-left.
    let location: NSPoint = NSEvent::mouseLocation();
    let screen_height = NSScreen::mainScreen(mtm)
      .map(|screen| screen.frame().size.height)
      .ok_or(Error::FailedToGetCursorPosition)?;

    Ok(PhysicalPosition::new(
      location.x,
      screen_height - location.y,
    ))
  }
}
