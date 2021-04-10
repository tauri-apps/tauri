// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(doc_cfg, feature(doc_cfg))]

pub use anyhow::Result;

#[cfg(feature = "codegen")]
mod codegen;

#[cfg(feature = "codegen")]
pub use codegen::context::CodegenContext;

/// Run all build time helpers for your Tauri Application.
///
/// The current helpers include the following:
/// * Generates a Windows Resource file when targeting Windows.
///
/// # Platforms
///
/// [`build()`] should be called inside of `build.rs` regardless of the platform:
/// * New helpers may target more platforms in the future.
/// * Platform specific code is handled by the helpers automatically.
/// * A build script is required in order to activate some cargo environmental variables that are
///   used when generating code and embedding assets - so [`build()`] may as well be called.
///
/// In short, this is saying don't put the call to [`build()`] behind a `#[cfg(windows)]`.
///
/// # Panics
///
/// If any of the build time helpers fail, they will [`std::panic!`] with the related error message.
/// This is typically desirable when running inside a build script; see [`try_build`] for no panics.
pub fn build() {
  if let Err(error) = try_build() {
    panic!("error found during tauri-build: {}", error);
  }
}

/// Non-panicking [`build()`].
pub fn try_build() -> Result<()> {
  #[cfg(windows)]
  {
    use anyhow::{anyhow, Context};
    use std::path::Path;
    use winres::WindowsResource;

    if Path::new("icons/icon.ico").exists() {
      let mut res = WindowsResource::new();
      res.set_icon_with_id("icons/icon.ico", "32512");
      res.compile().with_context(|| {
        "failed to compile icons/icon.ico into a Windows Resource file during tauri-build"
      })?;
    } else {
      return Err(anyhow!("no icons/icon.ico file found; required for generating a Windows Resource file during tauri-build"));
    }
  }

  Ok(())
}
