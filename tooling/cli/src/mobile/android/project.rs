// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::helpers::template;
use cargo_mobile::{
  android::{
    config::{Config, Metadata},
    env::Env,
    ndk,
    target::Target,
  },
  dot_cargo, os,
  target::TargetTrait as _,
  util::{
    self,
    cli::{Report, TextWrapper},
    ln, prefix_path,
  },
};
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};

use std::{
  ffi::OsStr,
  fs,
  path::{Path, PathBuf},
};

const TEMPLATE_DIR: Dir<'_> = include_dir!("templates/mobile/android");

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("failed to run rustup: {0}")]
  RustupFailed(bossy::Error),
  #[error("failed to process template: {0}")]
  TemplateProcessingFailed(String),
  #[error("failed to create directory at {path}: {cause}")]
  DirectoryCreationFailed {
    path: PathBuf,
    cause: std::io::Error,
  },
  #[error("failed to symlink asset directory")]
  AssetDirSymlinkFailed,
  #[error(transparent)]
  DotCargoGenFailed(ndk::MissingToolError),
  #[error("failed to copy {src} to {dest}: {cause}")]
  FileCopyFailed {
    src: PathBuf,
    dest: PathBuf,
    cause: std::io::Error,
  },
  #[error("asset source {0} is invalid")]
  AssetSourceInvalid(PathBuf),
}

pub fn gen(
  config: &Config,
  metadata: &Metadata,
  env: &Env,
  (handlebars, mut map): (Handlebars, template::JsonMap),
  wrapper: &TextWrapper,
  dot_cargo: &mut dot_cargo::DotCargo,
) -> Result<(), Error> {
  println!("Installing Android toolchains...");
  Target::install_all().map_err(Error::RustupFailed)?;
  println!("Generating Android Studio project...");
  let dest = config.project_dir();
  let asset_packs = metadata.asset_packs().unwrap_or_default();

  map.insert(
    "root-dir-rel",
    Path::new(&os::replace_path_separator(
      util::relativize_path(config.app().root_dir(), config.project_dir().join(config.app().name()))
        .into_os_string(),
    )),
  );
  map.insert("root-dir", config.app().root_dir());
  map.insert("targets", Target::all().values().collect::<Vec<_>>());
  map.insert("target-names", Target::all().keys().collect::<Vec<_>>());
  map.insert(
    "arches",
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
  map.insert(
    "asset-packs",
    asset_packs
      .iter()
      .map(|p| p.name.as_str())
      .collect::<Vec<_>>(),
  );
  map.insert("windows", cfg!(windows));

  let domain = config.app().reverse_domain().replace('.', "/");
  let package_path = format!("java/{}/{}", domain, config.app().name());

  let mut created_dirs = Vec::new();
  template::render_with_generator(
    &handlebars,
    map.inner(),
    &TEMPLATE_DIR,
    &dest,
    &mut |path| {
      let path = if path.extension() == Some(OsStr::new("kt")) {
        let parent = path.parent().unwrap();
        let file_name = path.file_name().unwrap();
        let out_dir = dest.join(parent).join(&package_path);
        out_dir.join(file_name)
      } else {
        dest.join(path)
      };

      let parent = path.parent().unwrap().to_path_buf();
      if !created_dirs.contains(&parent) {
        fs::create_dir_all(&parent)?;
        created_dirs.push(parent);
      }

      fs::File::create(path)
    },
  )
  .map_err(|e| Error::TemplateProcessingFailed(e.to_string()))?;

  if !asset_packs.is_empty() {
    Report::action_request(
      "When running from Android Studio, you must first set your deployment option to \"APK from app bundle\".",
      "Android Studio will not be able to find your asset packs otherwise. The option can be found under \"Run > Edit Configurations > Deploy\"."
    ).print(wrapper);
  }

  let source_dest = dest.join("app");
  for source in metadata.app_sources() {
    let source_src = config.app().root_dir().join(&source);
    let source_file = source_src
      .file_name()
      .ok_or_else(|| Error::AssetSourceInvalid(source_src.clone()))?;
    fs::copy(&source_src, source_dest.join(source_file)).map_err(|cause| {
      Error::FileCopyFailed {
        src: source_src,
        dest: source_dest.clone(),
        cause,
      }
    })?;
  }

  let dest = prefix_path(dest, "app/src/main/");
  fs::create_dir_all(&dest).map_err(|cause| Error::DirectoryCreationFailed {
    path: dest.clone(),
    cause,
  })?;
  os::ln::force_symlink_relative(config.app().asset_dir(), dest, ln::TargetStyle::Directory)
    .map_err(|_| Error::AssetDirSymlinkFailed)?;

  {
    for target in Target::all().values() {
      dot_cargo.insert_target(
        target.triple.to_owned(),
        target
          .generate_cargo_config(config, env)
          .map_err(Error::DotCargoGenFailed)?,
      );
    }
  }

  Ok(())
}
