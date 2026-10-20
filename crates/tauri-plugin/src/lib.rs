// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Interface for building Tauri plugins.
//!
//! A Tauri plugin is a Rust crate that extends a Tauri application with commands, state,
//! lifecycle hooks and - optionally - Android and iOS native code. This crate holds the pieces
//! of that interface that are not part of the [`tauri`] crate itself, split in two Cargo
//! features:
//!
//! - **`build`**: helpers for the plugin `build.rs`. [`Builder`] checks the crate against the
//!   Tauri plugin conventions (the `links` key must be set and match the crate name, the name
//!   cannot contain underscores or be a reserved one), autogenerates the
//!   `allow-$command`/`deny-$command` permissions for the plugin commands, parses the
//!   permission files, generates their JSON schema and reference documentation, defines the
//!   global scope schema and links the Android/iOS projects of the plugin.
//!   [`plugin_config`] reads the plugin configuration the Tauri CLI forwards to the build
//!   script, and the [`mobile`] module has helpers to patch the generated iOS `Info.plist` and
//!   entitlements and the Android manifest.
//! - **`runtime`**: reserved for the runtime side of the plugin interface. It currently
//!   exports nothing - use [`tauri::plugin`] to define the plugin itself.
//!
//! # Examples
//!
//! A typical plugin `build.rs`:
//!
//! ```rust,ignore
//! const COMMANDS: &[&str] = &["ping", "execute"];
//!
//! fn main() {
//!   tauri_plugin::Builder::new(COMMANDS)
//!     .android_path("android")
//!     .ios_path("ios")
//!     .build();
//! }
//! ```
//!
//! With a configuration type and an iOS `Info.plist` change:
//!
//! ```rust,ignore
//! #[derive(serde::Deserialize)]
//! #[serde(rename_all = "camelCase")]
//! struct Config {
//!   camera_usage_description: Option<String>,
//! }
//!
//! fn main() {
//!   if let Some(config) = tauri_plugin::plugin_config::<Config>("my-plugin") {
//!     if let Some(description) = config.camera_usage_description {
//!       tauri_plugin::mobile::update_info_plist(|plist| {
//!         plist.insert("NSCameraUsageDescription".into(), description.into());
//!       })
//!       .expect("failed to update Info.plist");
//!     }
//!   }
//!
//!   tauri_plugin::Builder::new(&["take_picture"]).build();
//! }
//! ```
//!
//! [`tauri`]: https://docs.rs/tauri/latest/tauri/
//! [`tauri::plugin`]: https://docs.rs/tauri/latest/tauri/plugin/index.html
//! [`Builder`]: https://docs.rs/tauri-plugin/latest/tauri_plugin/struct.Builder.html
//! [`plugin_config`]: https://docs.rs/tauri-plugin/latest/tauri_plugin/fn.plugin_config.html
//! [`mobile`]: https://docs.rs/tauri-plugin/latest/tauri_plugin/mobile/index.html

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "build")]
mod build;
#[cfg(feature = "runtime")]
mod runtime;

#[cfg(feature = "build")]
#[cfg_attr(docsrs, doc(cfg(feature = "build")))]
pub use build::*;
#[cfg(feature = "runtime")]
#[cfg_attr(docsrs, doc(cfg(feature = "runtime")))]
#[allow(unused)]
pub use runtime::*;
