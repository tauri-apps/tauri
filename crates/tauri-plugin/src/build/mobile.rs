// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Mobile-specific build utilities.

use std::{
  env::var_os,
  fs::{copy, create_dir, create_dir_all, read_to_string, remove_dir_all, write},
  path::{Path, PathBuf},
};

use anyhow::{Context, Result};

use super::{build_var, cfg_alias};

#[cfg(target_os = "macos")]
pub fn update_entitlements<F: FnOnce(&mut plist::Dictionary)>(f: F) -> Result<()> {
  if let (Some(project_path), Ok(app_name)) = (
    var_os("TAURI_IOS_PROJECT_PATH").map(PathBuf::from),
    std::env::var("TAURI_IOS_APP_NAME"),
  ) {
    update_plist_file(
      project_path
        .join(format!("{app_name}_iOS"))
        .join(format!("{app_name}_iOS.entitlements")),
      f,
    )?;
  }

  Ok(())
}

pub fn update_android_manifest(block_identifier: &str, parent: &str, insert: String) -> Result<()> {
  if let Some(project_path) = var_os("TAURI_ANDROID_PROJECT_PATH").map(PathBuf::from) {
    let manifest_path = project_path.join("app/src/main/AndroidManifest.xml");
    let manifest = read_to_string(&manifest_path)?;
    let rewritten = insert_into_xml(&manifest, block_identifier, parent, &insert);
    if rewritten != manifest {
      write(manifest_path, rewritten)?;
    }
  }
  Ok(())
}

pub(crate) fn setup(
  android_path: Option<PathBuf>,
  #[allow(unused_variables)] ios_path: Option<PathBuf>,
) -> Result<()> {
  let target_os = build_var("CARGO_CFG_TARGET_OS")?;
  let mobile = target_os == "android" || target_os == "ios";
  cfg_alias("mobile", mobile);
  cfg_alias("desktop", !mobile);

  match target_os.as_str() {
    "android" => {
      if let Some(path) = android_path {
        let manifest_dir = build_var("CARGO_MANIFEST_DIR").map(PathBuf::from)?;
        let source = manifest_dir.join(path);

        let tauri_library_path = std::env::var("DEP_TAURI_ANDROID_LIBRARY_PATH")
            .expect("missing `DEP_TAURI_ANDROID_LIBRARY_PATH` environment variable. Make sure `tauri` is a dependency of the plugin.");
        println!("cargo:rerun-if-env-changed=DEP_TAURI_ANDROID_LIBRARY_PATH");

        create_dir_all(source.join(".tauri")).context("failed to create .tauri directory")?;
        copy_folder(
          Path::new(&tauri_library_path),
          &source.join(".tauri").join("tauri-api"),
          &[],
        )
        .context("failed to copy tauri-api to the plugin project")?;

        println!("cargo:android_library_path={}", source.display());
      }
    }
    #[cfg(target_os = "macos")]
    "ios" => {
      if let Some(path) = ios_path {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
          .map(PathBuf::from)
          .unwrap();
        let tauri_library_path = std::env::var("DEP_TAURI_IOS_LIBRARY_PATH")
            .expect("missing `DEP_TAURI_IOS_LIBRARY_PATH` environment variable. Make sure `tauri` is a dependency of the plugin.");

        let tauri_dep_path = path.parent().unwrap().join(".tauri");
        create_dir_all(&tauri_dep_path).context("failed to create .tauri directory")?;
        copy_folder(
          Path::new(&tauri_library_path),
          &tauri_dep_path.join("tauri-api"),
          &[".build", "Package.resolved", "Tests"],
        )
        .context("failed to copy tauri-api to the plugin project")?;
        tauri_utils::build::link_apple_library(
          &std::env::var("CARGO_PKG_NAME").unwrap(),
          manifest_dir.join(path),
        );
      }
    }
    _ => (),
  }

  Ok(())
}

fn copy_folder(source: &Path, target: &Path, ignore_paths: &[&str]) -> Result<()> {
  let _ = remove_dir_all(target);

  for entry in walkdir::WalkDir::new(source) {
    let entry = entry?;
    let rel_path = entry.path().strip_prefix(source)?;
    let rel_path_str = rel_path.to_string_lossy();
    if ignore_paths
      .iter()
      .any(|path| rel_path_str.starts_with(path))
    {
      continue;
    }
    let dest_path = target.join(rel_path);

    if entry.file_type().is_dir() {
      create_dir(&dest_path)
        .with_context(|| format!("failed to create directory {}", dest_path.display()))?;
    } else {
      copy(entry.path(), &dest_path).with_context(|| {
        format!(
          "failed to copy {} to {}",
          entry.path().display(),
          dest_path.display()
        )
      })?;
      println!("cargo:rerun-if-changed={}", entry.path().display());
    }
  }

  Ok(())
}

#[cfg(target_os = "macos")]
fn update_plist_file<P: AsRef<Path>, F: FnOnce(&mut plist::Dictionary)>(
  path: P,
  f: F,
) -> Result<()> {
  use std::io::Cursor;

  let path = path.as_ref();
  if path.exists() {
    let plist_str = read_to_string(path)?;
    let mut plist = plist::Value::from_reader(Cursor::new(&plist_str))?;
    if let Some(dict) = plist.as_dictionary_mut() {
      f(dict);
      let mut plist_buf = Vec::new();
      let writer = Cursor::new(&mut plist_buf);
      plist::to_writer_xml(writer, &plist)?;
      let new_plist_str = String::from_utf8(plist_buf)?;
      if new_plist_str != plist_str {
        write(path, new_plist_str)?;
      }
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
      rewritten.push(format!("{}{}", indentation, block_comment));
      for l in contents.split('\n') {
        rewritten.push(format!("{}{}", indentation, l));
      }
      rewritten.push(format!("{}{}", indentation, block_comment));
    }

    rewritten.push(line.to_string());
  }

  rewritten.join("\n")
}

#[cfg(test)]
mod tests {
  #[test]
  fn insert_into_xml() {
    let manifest = r#"<manifest>
    <application>
        <intent-filter>
        </intent-filter>
    </application>
</manifest>"#;
    let id = "tauritest";
    let new = super::insert_into_xml(manifest, id, "application", "<something></something>");

    let block_id_comment = super::xml_block_comment(id);
    let expected = format!(
      r#"<manifest>
    <application>
        <intent-filter>
        </intent-filter>
        {block_id_comment}
        <something></something>
        {block_id_comment}
    </application>
</manifest>"#
    );

    assert_eq!(new, expected);

    // assert it's still the same after an empty update
    let new = super::insert_into_xml(&expected, id, "application", "<something></something>");
    assert_eq!(new, expected);
  }
}
