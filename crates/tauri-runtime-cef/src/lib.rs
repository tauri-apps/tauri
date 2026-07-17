// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

mod cef_impl;
mod external_message_pump;
mod platform;
mod runtime;
mod webview;
mod window;
mod window_builder;
mod window_handle;

pub use runtime::*;
pub use webview::*;
pub use window::CefWindowDispatcher;
pub use window_builder::WindowBuilderWrapper;

/// The CEF distribution version this crate links against (the download-cef cache
/// key, e.g. `150.0.10`), captured from `cef-dll-sys` by the build script. Used
/// to fetch the matching CEF for a prebuilt cef library. Empty if it could not
/// be determined at build time.
pub const CEF_VERSION: &str = env!("TAURI_RUNTIME_CEF_VERSION");
