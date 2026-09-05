// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

mod cef_impl;
mod devtools;
mod external_message_pump;
mod frame;
mod frame_navigation;
mod platform;
mod runtime;
mod webview;
pub use devtools::{DevToolsMessageIdExhausted, allocate_devtools_message_id};
pub use frame::{FrameEvent, FrameEventHandler, FrameEventKind};
pub use frame_navigation::{FrameNavigationState, NativeDocumentToken};
mod window;
mod window_builder;
mod window_handle;

pub use runtime::*;
pub use webview::*;
pub use window::{CefWindowDispatcher, NativeWindowToken};
pub use window_builder::WindowBuilderWrapper;
