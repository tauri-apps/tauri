// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(unused_variables)]

mod mock_runtime;
pub use mock_runtime::*;

#[cfg(shell_scope)]
use std::collections::HashMap;
use std::{borrow::Cow, sync::Arc};

#[cfg(shell_scope)]
use crate::ShellScopeConfig;
use crate::{Manager, Pattern};
use tauri_utils::{
  assets::{AssetKey, Assets, CspHash},
  config::{CliConfig, Config, PatternKind, TauriConfig},
};

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

pub fn noop_assets() -> NoopAsset {
  NoopAsset {
    csp_hashes: Default::default(),
  }
}

pub fn mock_context<A: Assets>(assets: A) -> crate::Context<A> {
  crate::Context {
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

pub fn mock_app() -> crate::App<MockRuntime> {
  crate::Builder::<MockRuntime>::new()
    .build(mock_context(noop_assets()))
    .unwrap()
}

pub(crate) fn mock_invoke_context() -> crate::endpoints::InvokeContext<MockRuntime> {
  let app = mock_app();
  crate::endpoints::InvokeContext {
    window: app.get_window("main").unwrap(),
    config: app.config(),
    package_info: app.package_info().clone(),
  }
}
