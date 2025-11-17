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
    .env("PATH", std::env::var_os("PATH").unwrap_or_default())
    .status()
    .unwrap();

  assert!(status.success());

  let lib_out_dir = derived_data_path
    .join("Build")
    .join("Products")
    .join(format!("{configuration}-{sdk}"));

  println!(
    "cargo::rustc-link-search=framework={}",
    lib_out_dir.display()
  );
  println!("cargo:rerun-if-changed={}", source.display());
  println!("cargo:rustc-link-search=native={}", lib_out_dir.display());
  println!("cargo:rustc-link-lib=static={name}");
}

/// Updates the Android manifest by inserting XML content into a specified parent tag.
///
/// The content is wrapped in auto-generated comments and will replace any existing
/// content with the same block identifier.
///
/// # Arguments
///
/// * `block_identifier` - A unique identifier for the block (used in comments)
/// * `parent` - The parent XML tag name (e.g., "activity", "application")
/// * `insert` - The XML content to insert
pub fn update_android_manifest(
  block_identifier: &str,
  parent: &str,
  insert: String,
) -> anyhow::Result<()> {
  use std::{
    env::var_os,
    fs::{read_to_string, write},
    path::PathBuf,
  };

  if let Some(project_path) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
    let manifest_path = project_path.join("app/src/main/AndroidManifest.xml");
    if !manifest_path.exists() {
      return Ok(());
    }
    let manifest = read_to_string(&manifest_path)?;
    let rewritten = insert_into_xml(&manifest, block_identifier, parent, &insert);
    if rewritten != manifest {
      write(&manifest_path, rewritten)?;
    }
  }
  Ok(())
}

fn xml_block_comment(id: &str) -> String {
  format!("<!-- {id}. AUTO-GENERATED. DO NOT REMOVE. -->")
}

fn insert_into_xml(xml: &str, block_identifier: &str, parent_tag: &str, contents: &str) -> String {
  let block_comment = xml_block_comment(block_identifier);

  let mut rewritten = Vec::new();
  let mut found_block = false;
  let parent_closing_tag = format!("</{parent_tag}>");
  for line in xml.split('\n') {
    if line.contains(&block_comment) {
      found_block = !found_block;
      continue;
    }

    // found previous block which should be removed
    if found_block {
      continue;
    }

    if let Some(index) = line.find(&parent_closing_tag) {
      let indentation = " ".repeat(index + 4);
      rewritten.push(format!("{indentation}{block_comment}"));
      for l in contents.split('\n') {
        rewritten.push(format!("{indentation}{l}"));
      }
      rewritten.push(format!("{indentation}{block_comment}"));
    }

    rewritten.push(line.to_string());
  }

  rewritten.join("\n")
}
