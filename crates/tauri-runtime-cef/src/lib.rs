// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

mod browser_client;
mod ipc;
mod platform;
mod request_context;
mod request_handler;
mod runtime;
mod webview;
mod window;
mod window_builder;

pub use runtime::*;
pub use webview::*;
pub use window::CefWindowDispatcher;
pub use window_builder::WindowBuilderWrapper;
