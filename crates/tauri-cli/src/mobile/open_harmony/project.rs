// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  error::{Context, ErrorExt},
  helpers::{config::Config as TauriConfig, template},
  Result,
};
use cargo_mobile2::{
  config::app::App,
  open_harmony::{config::Config, target::Target},
  os,
  target::TargetTrait,
  util,
};
use handlebars::Handlebars;
use include_dir::{include_dir, Dir};

use std::path::Path;

const TEMPLATE_DIR: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/templates/mobile/open-harmony");

pub fn gen(
  app: &App,
  tauri_config: &TauriConfig,
  config: &Config,
  (handlebars, mut map): (Handlebars, template::JsonMap),
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
      println!("Installing OpenHarmony Rust toolchains...");
      for target in missing_targets {
        target
          .install()
          .context("failed to install target with rustup")?;
      }
    }
  }

  println!("Generating DevEco Studio project...");
  let dest = config.project_dir();

  // Use the first configured PNG, as the default window icon does.
  let icon = tauri_config
    .bundle
    .icon
    .iter()
    .find(|icon| icon.ends_with(".png"))
    .map(|icon| {
      let path = app.root_dir().join(icon);
      std::fs::read(&path).fs_context("failed to read OpenHarmony app icon", path)
    })
    .transpose()?;

  map.insert(
    "app-icon",
    if icon.is_some() {
      "$media:app_icon"
    } else {
      "$media:layered_image"
    },
  );
  map.insert(
    "start-window-icon",
    if icon.is_some() {
      "$media:app_icon"
    } else {
      "$media:startIcon"
    },
  );

  map.insert(
    "root-dir-rel",
    Path::new(&os::replace_path_separator(
      util::relativize_path(app.root_dir(), dest.join("entry")).into_os_string(),
    )),
  );
  map.insert("root-dir", app.root_dir());
  map.insert("windows", cfg!(windows));

  template::render(&handlebars, map.inner(), &TEMPLATE_DIR, &dest)
    .with_context(|| "failed to process template")?;

  if let Some(icon) = icon {
    // AppScope resources are shared with the entry module.
    let path = dest.join("AppScope/resources/base/media/app_icon.png");
    std::fs::write(&path, icon).fs_context("failed to write OpenHarmony app icon", path)?;
  }

  Ok(())
}
