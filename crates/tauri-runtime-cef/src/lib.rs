// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

mod cef_impl;
mod devtools;
mod dialog;
mod external_message_pump;
mod frame;
pub use dialog::{
  NativeDialogKind, NativeDialogObservation, NativeDialogSnapshot, NativeDialogToken,
};
mod frame_navigation;
mod macros;
mod platform;
mod popup;
mod runtime;
// `LinuxSandboxPolicy` is public API on every platform and lives in `runtime`; only the
// decision logic behind it is Linux and BSD specific.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
mod sandbox;
mod tauri_ext;
mod webview;
pub use devtools::{DevToolsMessageIdExhausted, allocate_devtools_message_id};
pub use frame::{FrameEvent, FrameEventHandler, FrameEventKind};
pub use frame_navigation::{FrameNavigationState, NativeDocumentToken};
mod window;
mod window_builder;
mod window_handle;

pub use cef::sys::CEF_API_VERSION_LAST;
#[cfg(target_os = "macos")]
pub use platform::macos::setup_application as prepare_macos_application;
pub use runtime::*;
pub use tauri_ext::*;
/// Marks the application entry point so non-browser CEF processes (renderer, GPU, ...) are handled.
pub use tauri_macros::cef_entry_point;
pub use webview::*;
pub use window::{CefWindowDispatcher, NativeWindowToken};
pub use window_builder::WindowBuilderWrapper;
