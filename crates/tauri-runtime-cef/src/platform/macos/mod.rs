// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

mod application;
mod dock;
mod event_loop;
mod fullscreen;
mod monitor;
mod progress;
mod utils;
mod wake;
mod webview;
mod window;

pub use application::setup_application;
pub(crate) use application::{
  AppDelegate, AppDelegateEvent, activate_application, set_application_event_handler,
};
pub(crate) use fullscreen::FullscreenTransition;
pub(crate) use wake::MainThreadWake;
pub(crate) use window::nswindow;
