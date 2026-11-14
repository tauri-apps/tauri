// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! A safe Rust API over Qt Widgets and QtWebEngine, built on [`qtshim_sys`].
//!
//! It covers what a Tauri runtime and a menu library need, not all of Qt:
//!
//! - [`app`]: the `QApplication`, its event loop, the event channel and app-wide queries
//!   (screens, theme, cursor).
//! - [`menu`]: menubars, menus, context menus and standard actions.
//! - [`webengine`] (feature `webengine`): windows hosting `QWebEngineView`s, web profiles,
//!   custom URL schemes and IPC.
//! - [`test_harness`] (feature `test-harness`): runs each Qt test case in its own process, for
//!   `harness = false` integration tests of crates built on this one.
//!
//! # Safety and threading
//!
//! The whole API is safe. Qt is single-threaded: everything must run on the main thread (the one
//! that called [`app::init`]), between [`app::init`] and [`app::shutdown`]. This is checked at
//! run time: other calls panic (types that must stay on the main thread are also not `Send`).
//! [`app::dispatch`] and `webengine::Responder` may be used from any thread.
//!
//! Qt pointers coming from elsewhere (e.g. a `QWidget*` handed out by a Tauri runtime) enter
//! through [`menu::Widget::from_ptr`], which only accepts live widgets of this application.
//!
//! # Building
//!
//! Needs a C++17 compiler, `pkg-config` and the Qt 6 development files; see [`qtshim_sys`].
#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![warn(missing_docs)]

pub mod app;
pub mod menu;
#[cfg(feature = "webengine")]
pub mod webengine;

#[cfg(feature = "test-harness")]
pub mod test_harness;

mod lifecycle;
mod util;
