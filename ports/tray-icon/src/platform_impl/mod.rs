// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(target_os = "windows")]
#[path = "windows/mod.rs"]
mod platform;
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
))]
#[path = "gtk/mod.rs"]
mod platform;
#[cfg(all(target_os = "linux", not(feature = "linux-ksni")))]
#[path = "gtk/mod.rs"]
mod platform;
#[cfg(all(target_os = "linux", feature = "linux-ksni"))]
#[path = "linux/mod.rs"]
mod platform;
#[cfg(target_os = "macos")]
#[path = "macos/mod.rs"]
mod platform;

pub(crate) use self::platform::*;
