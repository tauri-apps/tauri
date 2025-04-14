// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Build script utilities.

/// Link a Swift library.
#[cfg(target_os = "macos")]
pub fn link_apple_library(name: &str, source: impl AsRef<std::path::Path>) {
  if source.as_ref().join("Package.swift").exists() {
    link_swift_library(name, source);
  } else {
    link_xcode_library(name, source);
  }
}

/// Link a Swift library.
#[cfg(target_os = "macos")]
fn link_swift_library(name: &str, source: impl AsRef<std::path::Path>) {
  let source = source.as_ref();

  let sdk_root = std::env::var_os("SDKROOT");
  std::env::remove_var("SDKROOT");

  swift_rs::SwiftLinker::new(
    &std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "10.13".into()),
  )
  .with_ios(&std::env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".into()))
  .with_package(name, source)
  .link();

  if let Some(root) = sdk_root {
    std::env::set_var("SDKROOT", root);
  }
}

/// Link a Xcode library.
#[cfg(target_os = "macos")]
fn link_xcode_library(name: &str, source: impl AsRef<std::path::Path>) {
  use std::{path::PathBuf, process::Command};

  let source = source.as_ref();
  let configuration = if std::env::var("DEBUG")
    .map(|v| v == "true")
    .unwrap_or_default()
  {
    "Debug"
  } else {
    "Release"
  };

  let (sdk, arch) = match std::env::var("TARGET").unwrap().as_str() {
    "aarch64-apple-ios" => ("iphoneos", "arm64"),
    "aarch64-apple-ios-sim" => ("iphonesimulator", "arm64"),
    "x86_64-apple-ios" => ("iphonesimulator", "x86_64"),
    _ => return,
  };

  let out_dir = std::env::var_os("OUT_DIR").map(PathBuf::from).unwrap();
  let derived_data_path = out_dir.join(format!("derivedData-{name}"));

  let status = Command::new("xcodebuild")
    .arg("build")
    .arg("-scheme")
    .arg(name)
    .arg("-configuration")
    .arg(configuration)
    .arg("-sdk")
    .arg(sdk)
    .arg("-arch")
    .arg(arch)
    .arg("-derivedDataPath")
    .arg(&derived_data_path)
    .arg("BUILD_LIBRARY_FOR_DISTRIBUTION=YES")
    .arg("OTHER_SWIFT_FLAGS=-no-verify-emitted-module-interface")
    .current_dir(source)
    .env_clear()
    .status()
    .unwrap();

  assert!(status.success());

  let lib_out_dir = derived_data_path
    .join("Build")
    .join("Products")
    .join(format!("{configuration}-{sdk}"));

  println!("cargo:rerun-if-changed={}", source.display());
  println!("cargo:rustc-link-search=native={}", lib_out_dir.display());
  println!("cargo:rustc-link-lib=static={name}");
}
