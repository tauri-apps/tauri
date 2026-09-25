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
  // FIXME: This can be accessed from multiple threads
  unsafe { std::env::remove_var("SDKROOT") };

  swift_rs::SwiftLinker::new(
    &std::env::var("MACOSX_DEPLOYMENT_TARGET").unwrap_or_else(|_| "10.13".into()),
  )
  .with_ios(&std::env::var("IPHONEOS_DEPLOYMENT_TARGET").unwrap_or_else(|_| "13.0".into()))
  .with_package(name, source)
  .link();

  if let Some(root) = sdk_root {
    // FIXME: This can be accessed from multiple threads
    unsafe { std::env::set_var("SDKROOT", root) };
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
/// content with the same block identifier. Empty content removes the block.
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

/// Removes the auto-generated block identified by `block_identifier` from the given XML string.
///
/// The block is delimited by the comments written by [`update_android_manifest`].
/// Returns the input unchanged when no such block exists.
pub fn remove_xml_block(xml: &str, block_identifier: &str) -> String {
  let block_comment = xml_block_comment(block_identifier);

  let mut rewritten = Vec::new();
  let mut in_block = false;
  for line in xml.split('\n') {
    if line.contains(&block_comment) {
      in_block = !in_block;
      continue;
    }
    if !in_block {
      rewritten.push(line);
    }
  }

  rewritten.join("\n")
}

fn insert_into_xml(xml: &str, block_identifier: &str, parent_tag: &str, contents: &str) -> String {
  let block_comment = xml_block_comment(block_identifier);
  let without_block = remove_xml_block(xml, block_identifier);

  // an empty block only removes the previously generated contents
  if contents.trim().is_empty() {
    return without_block;
  }

  let mut rewritten = Vec::new();
  let parent_closing_tag = format!("</{parent_tag}>");
  for line in without_block.split('\n') {
    if let Some(index) = line.find(&parent_closing_tag) {
      let indentation = " ".repeat(index + 4);
      rewritten.push(format!("{indentation}{block_comment}"));
      for l in contents.trim_end_matches('\n').split('\n') {
        rewritten.push(format!("{indentation}{l}"));
      }
      rewritten.push(format!("{indentation}{block_comment}"));
    }

    rewritten.push(line.to_string());
  }

  rewritten.join("\n")
}

#[cfg(test)]
mod tests {
  use super::{insert_into_xml, remove_xml_block};

  const MANIFEST: &str = r#"<manifest>
    <application>
        <activity android:name=".MainActivity" />
    </application>
</manifest>"#;

  #[test]
  fn inserts_block_before_parent_closing_tag() {
    let rewritten = insert_into_xml(MANIFEST, "test-block", "application", "<a />\n<b />\n");
    assert_eq!(
      rewritten,
      r#"<manifest>
    <application>
        <activity android:name=".MainActivity" />
        <!-- test-block. AUTO-GENERATED. DO NOT REMOVE. -->
        <a />
        <b />
        <!-- test-block. AUTO-GENERATED. DO NOT REMOVE. -->
    </application>
</manifest>"#
    );
  }

  #[test]
  fn replaces_existing_block() {
    let first = insert_into_xml(MANIFEST, "test-block", "application", "<a />");
    let second = insert_into_xml(&first, "test-block", "application", "<b />");
    assert!(!second.contains("<a />"));
    assert_eq!(second.matches("test-block").count(), 2);
    assert!(second.contains("<b />"));
  }

  #[test]
  fn empty_contents_removes_block() {
    let inserted = insert_into_xml(MANIFEST, "test-block", "application", "<a />");
    assert_eq!(
      insert_into_xml(&inserted, "test-block", "application", ""),
      MANIFEST
    );
    assert_eq!(remove_xml_block(&inserted, "test-block"), MANIFEST);
    assert_eq!(remove_xml_block(MANIFEST, "test-block"), MANIFEST);
  }
}
