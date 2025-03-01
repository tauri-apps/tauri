// Copyright 2016-2019 Cargo-Bundle developers <https://github.com/burtonageo/cargo-bundle>
// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This module provides utilities helping the packaging of desktop
//! applications for Linux:
//!
//! - Generation of [desktop entries] (`.desktop` files)
//! - Copy of icons in the [icons file hierarchy]
//!
//! The specifications are developed and hosted at [freedesktop.org].
//!
//! [freedesktop.org]: https://www.freedesktop.org
//! [desktop entries]: https://www.freedesktop.org/wiki/Specifications/desktop-entry-spec/
//! [icons file hierarchy]: https://specifications.freedesktop.org/icon-theme-spec/icon-theme-spec-latest.html#icon_lookup

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs::{read_to_string, File};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use anyhow::Context;
use handlebars::Handlebars;
use image::{self, codecs::png::PngDecoder, ImageDecoder};
use serde::Serialize;

use crate::{
  utils::{self, fs_utils},
  Settings,
};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct Icon {
  pub width: u32,
  pub height: u32,
  pub is_high_density: bool,
  pub path: PathBuf,
}

/// Generate the icon files, and returns a map where keys are the icons and
/// values are their current (source) path.
pub fn list_icon_files(
  settings: &Settings,
  data_dir: &Path,
) -> crate::Result<BTreeMap<Icon, PathBuf>> {
  let base_dir = data_dir.join("usr/share/icons/hicolor");
  let main_binary_name = settings.main_binary_name()?;
  let get_dest_path = |width: u32, height: u32, is_high_density: bool| {
    base_dir.join(format!(
      "{}x{}{}/apps/{}.png",
      width,
      height,
      if is_high_density { "@2" } else { "" },
      main_binary_name
    ))
  };
  let mut icons = BTreeMap::new();
  for icon_path in settings.icon_files() {
    let icon_path = icon_path?;
    if icon_path.extension() != Some(OsStr::new("png")) {
      continue;
    }
    // Put file in scope so that it's closed when copying it
    let icon = {
      let decoder = PngDecoder::new(BufReader::new(File::open(&icon_path)?))?;
      let width = decoder.dimensions().0;
      let height = decoder.dimensions().1;
      let is_high_density = utils::is_retina(&icon_path);
      let dest_path = get_dest_path(width, height, is_high_density);
      Icon {
        width,
        height,
        is_high_density,
        path: dest_path,
      }
    };
    icons.entry(icon).or_insert(icon_path);
  }

  Ok(icons)
}

/// Generate the icon files and store them under the `data_dir`.
pub fn copy_icon_files(settings: &Settings, data_dir: &Path) -> crate::Result<Vec<Icon>> {
  let icons = list_icon_files(settings, data_dir)?;
  for (icon, src) in &icons {
    fs_utils::copy_file(src, &icon.path)?;
  }

  Ok(icons.into_keys().collect())
}

/// Generate the application desktop file and store it under the `data_dir`.
/// Returns the path of the resulting file (source path) and the destination
/// path in the package.
pub fn generate_desktop_file(
  settings: &Settings,
  custom_template_path: &Option<PathBuf>,
  data_dir: &Path,
) -> crate::Result<(PathBuf, PathBuf)> {
  let bin_name = settings.main_binary_name()?;

  let product_name = settings.product_name();
  let desktop_file_name = format!("{product_name}.desktop");
  let path = PathBuf::from("usr/share/applications").join(desktop_file_name);
  let dest_path = PathBuf::from("/").join(&path);
  let file_path = data_dir.join(&path);
  let file = &mut fs_utils::create_file(&file_path)?;

  let mut handlebars = Handlebars::new();
  handlebars.register_escape_fn(handlebars::no_escape);
  if let Some(template) = custom_template_path {
    handlebars
      .register_template_string("main.desktop", read_to_string(template)?)
      .with_context(|| "Failed to setup custom handlebar template")?;
  } else {
    handlebars
      .register_template_string("main.desktop", include_str!("./main.desktop"))
      .with_context(|| "Failed to setup default handlebar template")?;
  }

  #[derive(Serialize)]
  struct DesktopTemplateParams<'a> {
    categories: &'a str,
    comment: Option<&'a str>,
    exec: &'a str,
    icon: &'a str,
    name: &'a str,
    mime_type: Option<String>,
    long_description: String,
  }

  let mut mime_type: Vec<String> = Vec::new();

  if let Some(associations) = settings.file_associations() {
    mime_type.extend(
      associations
        .iter()
        .filter_map(|association| association.mime_type.clone()),
    );
  }

  if let Some(protocols) = settings.deep_link_protocols() {
    mime_type.extend(
      protocols
        .iter()
        .flat_map(|protocol| &protocol.schemes)
        .map(|s| format!("x-scheme-handler/{s}")),
    );
  }

  let mime_type = (!mime_type.is_empty()).then_some(mime_type.join(";"));

  let bin_name_exec = if bin_name.contains(' ') {
    format!("\"{bin_name}\"")
  } else {
    bin_name.to_string()
  };

  handlebars.render_to_write(
    "main.desktop",
    &DesktopTemplateParams {
      categories: settings
        .app_category()
        .map(|app_category| app_category.freedesktop_categories())
        .unwrap_or(""),
      comment: if !settings.short_description().is_empty() {
        Some(settings.short_description())
      } else {
        None
      },
      exec: &bin_name_exec,
      icon: bin_name,
      name: settings.product_name(),
      mime_type,
      long_description: settings.long_description().unwrap_or_default().to_string(),
    },
    file,
  )?;

  Ok((file_path, dest_path))
}
