// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Utilities for unit testing on Tauri applications.
//!
//! # Stability
//!
//! This module is unstable.

#![allow(unused_variables)]

mod mock_runtime;
pub use mock_runtime::*;

#[cfg(shell_scope)]
use std::collections::HashMap;
use std::{borrow::Cow, sync::Arc};

#[cfg(shell_scope)]
use crate::ShellScopeConfig;
use crate::{App, Builder, Context, Pattern};
use tauri_utils::{
  assets::{AssetKey, Assets, CspHash},
  config::{CliConfig, Config, PatternKind, TauriConfig},
};

/// An empty [`Assets`] implementation.
pub struct NoopAsset {
  csp_hashes: Vec<CspHash<'static>>,
}

impl Assets for NoopAsset {
  fn get(&self, key: &AssetKey) -> Option<Cow<'_, [u8]>> {
    None
  }

  fn csp_hashes(&self, html_path: &AssetKey) -> Box<dyn Iterator<Item = CspHash<'_>> + '_> {
    Box::new(self.csp_hashes.iter().copied())
  }
}

/// Creates a new empty [`Assets`] implementation.
pub fn noop_assets() -> NoopAsset {
  NoopAsset {
    csp_hashes: Default::default(),
  }
}

/// Creates a new [`crate::Context`] for testing.
pub fn mock_context<A: Assets>(assets: A) -> crate::Context<A> {
  Context {
    config: Config {
      schema: None,
      package: Default::default(),
      tauri: TauriConfig {
        pattern: PatternKind::Brownfield,
        windows: vec![Default::default()],
        cli: Some(CliConfig {
          description: None,
          long_description: None,
          before_help: None,
          after_help: None,
          args: None,
          subcommands: None,
        }),
        bundle: Default::default(),
        allowlist: Default::default(),
        security: Default::default(),
        updater: Default::default(),
        system_tray: None,
        macos_private_api: false,
      },
      build: Default::default(),
      plugins: Default::default(),
    },
    assets: Arc::new(assets),
    default_window_icon: None,
    app_icon: None,
    system_tray_icon: None,
    package_info: crate::PackageInfo {
      name: "test".into(),
      version: "0.1.0".parse().unwrap(),
      authors: "Tauri",
      description: "Tauri test",
    },
    _info_plist: (),
    pattern: Pattern::Brownfield(std::marker::PhantomData),
    #[cfg(shell_scope)]
    shell_scope: ShellScopeConfig {
      open: None,
      scopes: HashMap::new(),
    },
  }
}

/// Creates a new [`Builder`] using the [`MockRuntime`].
///
/// To use a dummy [`Context`], see [`mock_app`].
///
/// # Examples
///
/// ```rust
/// //#[cfg(test)]
/// fn do_something() {
///   let app = tauri::test::mock_builder()
///     // remove the string argument to use your app's config file
///     .build(tauri::generate_context!("test/fixture/src-tauri/tauri.conf.json"))
///     .unwrap();
/// }
/// ```
pub fn mock_builder() -> Builder<MockRuntime> {
  Builder::<MockRuntime>::new()
}

/// Creates a new [`App`] for testing using the [`mock_context`] with a [`noop_assets`].
pub fn mock_app() -> App<MockRuntime> {
  mock_builder().build(mock_context(noop_assets())).unwrap()
}

#[cfg(test)]
pub(crate) fn mock_invoke_context() -> crate::endpoints::InvokeContext<MockRuntime> {
  use crate::Manager;
  let app = mock_app();
  crate::endpoints::InvokeContext {
    window: app.get_window("main").unwrap(),
    config: app.config(),
    package_info: app.package_info().clone(),
  }
}
