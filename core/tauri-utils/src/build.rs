// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Build script utilities.

/// Link a Swift library.
#[cfg(target_os = "macos")]
pub fn link_swift_library(name: &str, source: impl AsRef<std::path::Path>) {
  use std::{path::PathBuf, process::Command};

  let source = source.as_ref();

  let (sdk, platform) = match std::env::var("TARGET").unwrap().as_str() {
    "aarch64-apple-ios" => ("iphoneos", "iOS"),
    "aarch64-apple-ios-sim" => ("iphoneos", "iOS Simulator"),
    "x86_64-apple-ios" => ("iphonesimulator", "iOS Simulator"),
    _ => return,
  };

  let out_dir = std::env::var_os("OUT_DIR").map(PathBuf::from).unwrap();
  let archive_path = out_dir.join("out.xcarchive");
  let status = Command::new("xcodebuild")
    .current_dir(source)
    .arg("archive")
    .arg("-scheme")
    .arg(name)
    .arg("-archivePath")
    .arg(&archive_path)
    .arg("-sdk")
    .arg(sdk)
    .arg("-destination")
    .arg(format!("generic/platform={platform}"))
    .arg("BUILD_LIBRARY_FOR_DISTRIBUTION=YES")
    .arg("SKIP_INSTALL=NO")
    .arg("OTHER_SWIFT_FLAGS=-no-verify-emitted-module-interface")
    .status()
    .unwrap();

  assert!(status.success());

  let lib_out_dir = out_dir.join("__lib");
  std::fs::create_dir_all(&lib_out_dir).unwrap();

  let lib_path = lib_out_dir.join(format!("lib{name}.a"));

  if !archive_path.exists() {
    panic!("failed to archive");
  }

  let status = Command::new("/usr/bin/libtool")
    .arg("-static")
    //.arg("-arch_only")
    //.arg("arm64")
    .arg(format!("{name}.o"))
    .arg("-o")
    .arg(lib_path)
    .current_dir(&archive_path.join("Products/Users/lucas/Objects"))
    .status()
    .unwrap();
  assert!(status.success());

  println!("cargo:rerun-if-changed={}", source.display());
  println!("cargo:rustc-link-search=native={}", lib_out_dir.display());
  println!("cargo:rustc-link-lib=static={name}");
}
