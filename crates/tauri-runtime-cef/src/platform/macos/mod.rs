// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod application;
mod dock;
mod event_loop;
mod monitor;
mod progress;
mod utils;
mod webview;
mod window;

pub use application::setup_application;
pub(crate) use application::{
  AppDelegate, AppDelegateEvent, activate_application, set_application_event_handler,
};
