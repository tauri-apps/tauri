// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The webview runtime an application links, and everything the CLI does differently because of it.
//!
//! The runtime is selected by depending on its crate (`tauri-runtime-wry` or `tauri-runtime-cef`),
//! so it is detected from the app manifest. Every runtime-specific behavior (system package dependencies,
//! installer steps, code signing entitlements, what to ship with the bundle, how to run the app in dev mode)
//! is defined here, so the rest of the CLI only asks the [`Runtime`] what it needs.

use std::path::{Path, PathBuf};

use tauri_bundler::WebviewRuntime;
use tauri_utils::config::WebviewInstallMode;

use crate::interface::rust::manifest::Manifest;

pub mod cef;
#[cfg(target_os = "macos")]
pub mod macos_dev;

/// The `tauri-runtime-wry` crate.
pub const WRY_CRATE_NAME: &str = "tauri-runtime-wry";

/// The webview runtime linked into the application.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Runtime {
  /// `tauri-runtime-wry`: the system webview (webkit2gtk on Linux, WebView2 on Windows, WKWebView on Apple platforms).
  Wry,
  /// `tauri-runtime-cef`: the Chromium Embedded Framework, shipped with the application.
  Cef,
  /// Neither runtime crate is linked (a custom runtime). The CLI does nothing runtime-specific.
  Other,
}

/// A shared library the application loads at run time on Linux.
#[cfg(target_os = "linux")]
#[derive(Debug, Clone, Copy)]
pub struct LinuxDependency {
  /// The Debian package that provides the library.
  pub deb_package: &'static str,
  /// The library file name, as required by RPM packages.
  pub library: &'static str,
}

impl Runtime {
  /// Detects the runtime linked into the application for the given target and enabled features.
  ///
  /// See [`Manifest::uses_dependency`] for how a crate is considered linked.
  /// CEF takes precedence when both crates are linked since it is the one that must be shipped with the app.
  pub fn detect(manifest: &Manifest, enabled_features: &[String], target_triple: &str) -> Self {
    if manifest.uses_dependency(cef::CRATE_NAME, enabled_features, target_triple) {
      Self::Cef
    } else if manifest.uses_dependency(WRY_CRATE_NAME, enabled_features, target_triple) {
      Self::Wry
    } else {
      Self::Other
    }
  }

  /// The Linux shared libraries the application loads at run time,
  /// to be declared as dependencies of the Debian and RPM packages.
  #[cfg(target_os = "linux")]
  pub fn linux_dependencies(self) -> &'static [LinuxDependency] {
    match self {
      Self::Wry => &[LinuxDependency {
        deb_package: "libwebkit2gtk-4.1-0",
        library: "libwebkit2gtk-4.1.so.0",
      }],
      // CEF is shipped with the app
      Self::Cef | Self::Other => &[],
    }
  }

  /// The WebView2 installation step of the Windows installers.
  ///
  /// Only wry uses WebView2, so the configured mode is replaced by [`WebviewInstallMode::Skip`] for the other runtimes.
  pub fn webview_install_mode(self, configured: WebviewInstallMode) -> WebviewInstallMode {
    match self {
      Self::Wry => configured,
      Self::Cef | Self::Other => WebviewInstallMode::Skip,
    }
  }

  /// The minimum WebView2 version enforced by the Windows installers. Only meaningful for wry.
  pub fn minimum_webview2_version(self, configured: Option<String>) -> Option<String> {
    match self {
      Self::Wry => configured,
      Self::Cef | Self::Other => None,
    }
  }

  /// The entitlements the runtime needs when the application is code signed with the hardened runtime on macOS.
  #[cfg(target_os = "macos")]
  pub fn macos_entitlements(self) -> plist::Dictionary {
    let mut entitlements = plist::Dictionary::new();
    match self {
      Self::Wry | Self::Other => {}
      // Chromium's JIT and the unsigned CEF framework are rejected by the hardened runtime otherwise
      Self::Cef => {
        entitlements.insert("com.apple.security.cs.allow-jit".to_string(), true.into());
        entitlements.insert(
          "com.apple.security.cs.allow-unsigned-executable-memory".to_string(),
          true.into(),
        );
        entitlements.insert(
          "com.apple.security.cs.disable-library-validation".to_string(),
          true.into(),
        );
      }
    }
    entitlements
  }

  /// Whether `tauri dev` on macOS must run the application from inside an `.app` bundle.
  ///
  /// CEF launches its helper apps by path from inside the bundle, so the app cannot run as a bare executable.
  #[cfg(target_os = "macos")]
  pub fn macos_dev_in_app_bundle(self) -> bool {
    match self {
      Self::Cef => true,
      Self::Wry | Self::Other => false,
    }
  }

  /// The runtime as the bundler sees it, resolving the CEF distribution to ship.
  ///
  /// `embed_cef` is `bundle > cef > embed`: `false` for an app that loads CEF at run time
  /// from outside its bundle, so nothing of the distribution ships.
  pub fn bundler_runtime(
    self,
    embed_cef: bool,
    target: &str,
    workspace_dir: &Path,
  ) -> crate::Result<WebviewRuntime> {
    Ok(match self {
      Self::Wry => WebviewRuntime::Wry,
      Self::Other => WebviewRuntime::Other,
      // An app on a shared runtime links CEF but ships none: there may not
      // even be a distribution on this machine to resolve, so don't look.
      Self::Cef if !embed_cef => WebviewRuntime::Cef { distribution: None },
      Self::Cef => {
        let cef_path = std::env::var_os("CEF_PATH")
          .map(PathBuf::from)
          .unwrap_or_else(cef::default_path);
        WebviewRuntime::Cef {
          distribution: Some(cef::resolve_path_for_bundle(
            cef_path,
            target,
            workspace_dir,
          )?),
        }
      }
    })
  }
}
