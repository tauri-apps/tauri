// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  ffi::c_void,
  sync::{Arc, RwLock},
};

use objc2_app_kit::NSWindow;
use objc2_foundation::NSPoint;

use super::utils;

#[derive(Debug, Default)]
pub(crate) struct AppkitState {
  pub(crate) traffic_light_position: Option<NSPoint>,
}

impl AppkitState {
  pub(super) fn associate(state: &Arc<RwLock<Self>>, nswindow: &NSWindow) {
    utils::set_associated_data(nswindow, Self::key(), Arc::as_ptr(state));
  }

  pub(super) fn from_window(nswindow: &NSWindow) -> Option<&RwLock<Self>> {
    unsafe { utils::associated_data(nswindow, Self::key()) }
  }

  fn key() -> *const c_void {
    static APPKIT_STATE_KEY: u8 = 0;
    &APPKIT_STATE_KEY as *const u8 as *const c_void
  }
}
