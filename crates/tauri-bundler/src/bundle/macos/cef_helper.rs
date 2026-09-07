// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The executable of the macOS CEF helper apps, built at bundle time.
//!
//! CEF runs its renderer, GPU and utility processes from helper apps inside
//! the bundle. Their executable is a small Rust crate whose source the bundler
//! carries (`cef-helper/` in the crate root) and compiles here with cargo, for
//! the target being bundled only, pinned to the resolved `cef` and `cef-dll-sys` crate sources the app
//! links. The build runs with the `CEF_PATH` the app was built with, so
//! `cef-dll-sys` resolves the distribution it already downloaded there
//! instead of fetching another, and the helper loads the very framework the
//! app ships.

use crate::{
  Error::GenericError,
  Settings,
  bundle::settings::{Arch, CefHelperSettings},
  error::{Context, ErrorExt},
  utils::CommandExt,
};

use std::{
  fs,
  path::{Path, PathBuf},
  process::Command,
};

const MANIFEST_TEMPLATE: &str = include_str!("../../../cef-helper/Cargo.toml.in");
const MAIN_SOURCE: &str = include_str!("../../../cef-helper/main.rs");
const CEF_PATH_PLACEHOLDER: &str = "@CEF_PATH@";
const CEF_DLL_SYS_PATH_PLACEHOLDER: &str = "@CEF_DLL_SYS_PATH@";
const BIN_NAME: &str = "tauri-cef-helper";

/// Builds the helper executable for the bundle's architecture and returns
/// its path. A universal bundle gets both Apple targets built and `lipo`ed
/// together.
pub(super) fn build(settings: &Settings) -> crate::Result<PathBuf> {
  let helper_settings = settings
    .webview_runtime()
    .cef_helper()
    .ok_or_else(|| {
      GenericError(
        "the CEF helper apps' executable is compiled at bundle time, but the build is not configured (the `helper` of the `WebviewRuntime::Cef` bundle setting)"
          .into(),
      )
    })?;

  let targets: &[&str] = match settings.binary_arch() {
    Arch::AArch64 => &["aarch64-apple-darwin"],
    Arch::X86_64 => &["x86_64-apple-darwin"],
    Arch::Universal => &["aarch64-apple-darwin", "x86_64-apple-darwin"],
    other => {
      return Err(GenericError(format!(
        "CEF helper apps are only supported for aarch64, x86_64 and universal on macOS (got {other:?})"
      )));
    }
  };

  let crate_dir = write_crate(helper_settings)?;
  let target_dir = crate_dir.join("target");

  let binaries = targets
    .iter()
    .map(|target| cargo_build(&crate_dir, &target_dir, target, helper_settings))
    .collect::<crate::Result<Vec<_>>>()?;

  if let [binary] = binaries.as_slice() {
    return Ok(binary.clone());
  }

  let universal_dir = target_dir.join(settings.target()).join("release");
  fs::create_dir_all(&universal_dir).fs_context(
    "failed to create the universal CEF helper directory",
    &universal_dir,
  )?;
  let universal = universal_dir.join(BIN_NAME);
  Command::new("lipo")
    .arg("-create")
    .arg("-output")
    .arg(&universal)
    .args(&binaries)
    .output_ok()
    .with_context(|| "failed to create universal CEF helper binary using lipo")?;

  Ok(universal)
}

/// Lays the helper crate out in its build directory: the manifest with the
/// app's resolved CEF sources pinned, and `src/main.rs`.
///
/// Files are only rewritten when their contents changed so cargo's
/// fingerprints stay valid and a bundle after an unchanged one rebuilds nothing.
fn write_crate(helper_settings: &CefHelperSettings) -> crate::Result<PathBuf> {
  let crate_dir = helper_settings.build_dir.clone();
  let manifest = helper_manifest(helper_settings)?;
  write_if_changed(&crate_dir.join("Cargo.toml"), &manifest)?;
  write_if_changed(&crate_dir.join("src").join("main.rs"), MAIN_SOURCE)?;
  Ok(crate_dir)
}

fn helper_manifest(settings: &CefHelperSettings) -> crate::Result<String> {
  // Cargo paths come from its resolved metadata. JSON string escaping is also
  // valid for TOML basic strings, including Windows paths and quoted directories.
  Ok(
    MANIFEST_TEMPLATE
      .replace(
        CEF_PATH_PLACEHOLDER,
        &serde_json::to_string(&settings.cef_crate_path)?,
      )
      .replace(
        CEF_DLL_SYS_PATH_PLACEHOLDER,
        &serde_json::to_string(&settings.cef_dll_sys_crate_path)?,
      ),
  )
}

fn write_if_changed(path: &Path, contents: &str) -> crate::Result<()> {
  if fs::read_to_string(path).is_ok_and(|current| current == contents) {
    return Ok(());
  }
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).fs_context("failed to create CEF helper crate directory", parent)?;
  }
  fs::write(path, contents).fs_context("failed to write CEF helper crate file", path)
}

/// Runs `cargo build --release` for `target` and returns the executable's path.
fn cargo_build(
  crate_dir: &Path,
  target_dir: &Path,
  target: &str,
  helper_settings: &CefHelperSettings,
) -> crate::Result<PathBuf> {
  log::info!(action = "Building"; "CEF helper for {target}");

  // The cargo that is running us, when `cargo tauri` is: the toolchain the
  // app was built with. The one on PATH otherwise; through rustup that is
  // still the project's toolchain, resolved from the working directory.
  let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
  let mut command = Command::new(&cargo);
  command
    .arg("build")
    .arg("--release")
    .arg("--manifest-path")
    .arg(crate_dir.join("Cargo.toml"))
    .arg("--bin")
    .arg(BIN_NAME)
    .arg("--target")
    .arg(target)
    .env("CARGO_TARGET_DIR", target_dir)
    .env("CEF_PATH", &helper_settings.cef_path);

  // Cargo's own output goes straight to the terminal: the first build
  // compiles the `cef` crate and its C++ wrapper, which takes a while.
  let status = command
    .piped()
    .map_err(|error| crate::Error::CommandFailed {
      command: cargo.to_string_lossy().into_owned(),
      error,
    })?;
  if !status.success() {
    return Err(GenericError(format!(
      "failed to build the CEF helper for {target} ({status})"
    )));
  }

  Ok(target_dir.join(target).join("release").join(BIN_NAME))
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn helper_uses_resolved_sources_instead_of_framework_version() {
    let settings = CefHelperSettings {
      cef_crate_path: PathBuf::from("/cache/custom cef/cef"),
      cef_dll_sys_crate_path: PathBuf::from("/cache/custom cef/sys"),
      ..Default::default()
    };
    let manifest = helper_manifest(&settings).unwrap();
    assert!(manifest.contains(r#"cef = { path = "/cache/custom cef/cef""#));
    assert_eq!(
      manifest
        .matches(r#"path = "/cache/custom cef/sys""#)
        .count(),
      2
    );
    assert!(manifest.contains("[patch.crates-io]"));
    assert!(!manifest.contains("@CEF_"));
    assert!(!manifest.contains(r#"version = "=151"#));
  }
}
