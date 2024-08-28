// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{helpers::template, Result};
use anyhow::Context;
use cargo_mobile2::{
  android::{
    config::{Config, Metadata},
    target::Target,
  },
  config::app::DEFAULT_ASSET_DIR,
  os,
  target::TargetTrait as _,
  util::{
    self,
    cli::{Report, TextWrapper},
    prefix_path,
  },
};
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};

use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
};

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/mobile/android");

pub fn gen(
  config: &Config,
  metadata: &Metadata,
  (handlebars, mut map): (Handlebars, template::JsonMap),
  wrapper: &TextWrapper,
  skip_targets_install: bool,
) -> Result<()> {
  if !skip_targets_install {
    let installed_targets =
      crate::interface::rust::installation::installed_targets().unwrap_or_default();
    let missing_targets = Target::all()
      .values()
      .filter(|t| !installed_targets.contains(&t.triple().into()))
      .collect::<Vec<&Target>>();

    if !missing_targets.is_empty() {
      println!("Installing Android Rust toolchains...");
      for target in missing_targets {
        target
          .install()
          .context("failed to install target with rustup")?;
      }
    }
  }
  println!("Generating Android Studio project...");
  let dest = config.project_dir();
  let asset_packs = metadata.asset_packs().unwrap_or_default();

  map.insert(
    "root-dir-rel",
    Path::new(&os::replace_path_separator(
      util::relativize_path(
        config.app().root_dir(),
        config.project_dir().join(config.app().name_snake()),
      )
      .into_os_string(),
    )),
  );
  map.insert("root-dir", config.app().root_dir());
  map.insert(
    "abi-list",
    Target::all()
      .values()
      .map(|target| target.abi)
      .collect::<Vec<_>>(),
  );
  map.insert("target-list", Target::all().keys().collect::<Vec<_>>());
  map.insert(
    "arch-list",
    Target::all()
      .values()
      .map(|target| target.arch)
      .collect::<Vec<_>>(),
  );
  map.insert("android-app-plugins", metadata.app_plugins());
  map.insert(
    "android-project-dependencies",
    metadata.project_dependencies(),
  );
  map.insert("android-app-dependencies", metadata.app_dependencies());
  map.insert(
    "android-app-dependencies-platform",
    metadata.app_dependencies_platform(),
  );
  map.insert(
    "has-code",
    metadata.project_dependencies().is_some()
      || metadata.app_dependencies().is_some()
      || metadata.app_dependencies_platform().is_some(),
  );
  map.insert("has-asset-packs", !asset_packs.is_empty());
  map.insert(
    "asset-packs",
    asset_packs
      .iter()
      .map(|p| p.name.as_str())
      .collect::<Vec<_>>(),
  );
  map.insert("windows", cfg!(windows));

  let identifier = config.app().identifier().replace('.', "/");
  let package_path = format!("java/{}", identifier);

  map.insert("package-path", &package_path);

  let mut created_dirs = Vec::new();
  template::render_with_generator(
    &handlebars,
    map.inner(),
    &TEMPLATE_DIR,
    &dest,
    &mut |path| generate_out_file(&path, &dest, &package_path, &mut created_dirs),
  )
  .with_context(|| "failed to process template")?;

  if !asset_packs.is_empty() {
    Report::action_request(
      "When running from Android Studio, you must first set your deployment option to \"APK from app bundle\".",
      "Android Studio will not be able to find your asset packs otherwise. The option can be found under \"Run > Edit Configurations > Deploy\"."
    ).print(wrapper);
  }

  let source_dest = dest.join("app");
  for source in metadata.app_sources() {
    let source_src = config.app().root_dir().join(source);
    let source_file = source_src
      .file_name()
      .ok_or_else(|| anyhow::anyhow!("asset source {} is invalid", source_src.display()))?;
    fs::copy(&source_src, source_dest.join(source_file)).map_err(|cause| {
      anyhow::anyhow!(
        "failed to copy {} to {}: {}",
        source_src.display(),
        source_dest.display(),
        cause
      )
    })?;
  }

  let dest = prefix_path(dest, "app/src/main/");
  fs::create_dir_all(&dest).map_err(|cause| {
    anyhow::anyhow!(
      "failed to create directory at {}: {}",
      dest.display(),
      cause
    )
  })?;

  let asset_dir = dest.join(DEFAULT_ASSET_DIR);
  if !asset_dir.is_dir() {
    fs::create_dir_all(&asset_dir).map_err(|cause| {
      anyhow::anyhow!(
        "failed to create asset dir {path}: {cause}",
        path = asset_dir.display()
      )
    })?;
  }

  Ok(())
}

fn generate_out_file(
  path: &Path,
  dest: &Path,
  package_path: &str,
  created_dirs: &mut Vec<PathBuf>,
) -> std::io::Result<Option<fs::File>> {
  let mut iter = path.iter();
  let root = iter.next().unwrap().to_str().unwrap();
  let path_without_root: std::path::PathBuf = iter.collect();
  let path = match (
    root,
    path.extension().and_then(|o| o.to_str()),
    path_without_root.strip_prefix("src/main"),
  ) {
    ("app" | "buildSrc", Some("kt"), Ok(path)) => {
      let parent = path.parent().unwrap();
      let file_name = path.file_name().unwrap();
      let out_dir = dest
        .join(root)
        .join("src/main")
        .join(package_path)
        .join(parent);
      out_dir.join(file_name)
    }
    _ => dest.join(path),
  };

  let parent = path.parent().unwrap().to_path_buf();
  if !created_dirs.contains(&parent) {
    fs::create_dir_all(&parent)?;
    created_dirs.push(parent);
  }

  let mut options = fs::OpenOptions::new();
  options.write(true);

  #[cfg(unix)]
  if path.file_name().unwrap() == OsStr::new("gradlew") {
    use std::os::unix::fs::OpenOptionsExt;
    options.mode(0o755);
  }

  if path.file_name().unwrap() == OsStr::new("BuildTask.kt") {
    options.truncate(true).create(true).open(path).map(Some)
  } else if !path.exists() {
    options.create(true).open(path).map(Some)
  } else {
    Ok(None)
  }
}
