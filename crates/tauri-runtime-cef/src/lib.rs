// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

mod cef_impl;
mod devtools;
mod dialog;
/// The diagnostic environment variables Chromium and CEF read, and the policy over them.
mod environment;
mod external_message_pump;
mod frame;
pub use dialog::{
  NativeDialogKind, NativeDialogObservation, NativeDialogSnapshot, NativeDialogToken,
};
pub use environment::DebugEnvironment;
mod frame_navigation;
/// The languages the user asked their operating system for.
mod locale;
mod macros;
mod platform;
mod popup;
mod runtime;
// `SandboxPolicy` itself is public API and lives in `runtime`; this module holds the
// decision behind it.
mod sandbox;
/// Helpers for the Chromium command line the runtime hands to CEF.
mod switches;
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
