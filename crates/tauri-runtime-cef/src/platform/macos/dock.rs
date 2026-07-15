// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::time::{Duration, Instant};

use objc2::{msg_send, runtime::AnyObject, sel};
use objc2_app_kit::{NSApplicationActivationOptions, NSRunningApplication};
use objc2_application_services::{
  kProcessTransformToForegroundApplication, kProcessTransformToUIElementApplication,
};
use objc2_foundation::NSString;

use super::{application::CefWinitApplication, utils};

const DOCK_SHOW_TIMEOUT: Duration = Duration::from_secs(1);
const DOCK_BUNDLE_IDENTIFIER: &str = "com.apple.dock";

impl CefWinitApplication {
  pub fn set_dock_visibility(&self, visible: bool) {
    if visible {
      self.set_dock_show();
    } else {
      self.set_dock_hide();
    }
  }

  fn set_dock_hide(&self) {
    let now = Instant::now();
    if let Some(last_dock_show_time) = self.last_dock_show() {
      // TransformProcessType from UIElement back to foreground is asynchronous
      // and does not expose a completion signal. Electron found that rapid
      // hide/show cycles can race the macOS Dock and leave duplicate app icons
      // behind, so it ignores hide requests immediately after showing.
      // https://github.com/electron/electron/blob/88cd4b418618424fbcd11917fffee489f534ad72/shell/browser/browser_mac.mm#L2376-L2408
      if now.duration_since(last_dock_show_time) < DOCK_SHOW_TIMEOUT {
        return;
      }
    }

    self.set_windows_can_hide(false);
    utils::transform_process_type(kProcessTransformToUIElementApplication);
  }

  fn set_dock_show(&self) {
    self.set_last_dock_show(Instant::now());
    self.set_windows_can_hide(true);

    if NSRunningApplication::currentApplication().isActive() {
      // TransformProcessType is buggy when bringing an active UIElement app
      // back to foreground. Electron works around it by activating Dock first,
      // then delaying the foreground transform and app reactivation:
      // https://github.com/electron/electron/blob/88cd4b418618424fbcd11917fffee489f534ad72/shell/browser/browser_mac.mm#L2424-L2475
      activate_dock();
      self.perform_delayed_dock_show();
    } else {
      utils::transform_process_type(kProcessTransformToForegroundApplication);
    }
  }

  fn set_windows_can_hide(&self, can_hide: bool) {
    let windows = self.windows();
    for idx in 0..windows.count() {
      windows.objectAtIndex(idx).setCanHide(can_hide);
    }
  }

  fn perform_delayed_dock_show(&self) {
    unsafe {
      let _: () = msg_send![
        self,
        performSelector: sel!(tauriTransformProcessToForeground),
        withObject: None::<&AnyObject>,
        afterDelay: 1.0f64,
      ];
      let _: () = msg_send![
        self,
        performSelector: sel!(tauriActivateCurrentApplication),
        withObject: None::<&AnyObject>,
        afterDelay: 2.0f64,
      ];
    }
  }
}

fn activate_dock() {
  let dock_id = NSString::from_str(DOCK_BUNDLE_IDENTIFIER);
  let dock_apps = NSRunningApplication::runningApplicationsWithBundleIdentifier(&dock_id);
  if dock_apps.count() > 0 {
    let app = dock_apps.objectAtIndex(0);
    #[allow(deprecated)]
    app.activateWithOptions(NSApplicationActivationOptions::ActivateIgnoringOtherApps);
  }
}
