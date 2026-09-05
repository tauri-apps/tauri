// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

mod cef_impl;
mod external_message_pump;
mod platform;
mod runtime;
mod tauri_ext;
mod webview;
mod window;
mod window_builder;
mod window_handle;

pub use cef::sys::CEF_API_VERSION_LAST;
pub use runtime::*;
pub use tauri_ext::*;
/// Marks the application entry point so non-browser CEF processes (renderer, GPU, ...) are handled.
pub use tauri_macros::cef_entry_point;
pub use webview::*;
pub use window::CefWindowDispatcher;
pub use window_builder::WindowBuilderWrapper;
