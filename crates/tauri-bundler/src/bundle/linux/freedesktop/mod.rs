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
use std::fs::{File, read_to_string};
use std::io::BufReader;
use std::path::{Path, PathBuf};

use handlebars::Handlebars;
use image::{self, ImageDecoder, codecs::png::PngDecoder};
use serde::Serialize;

use crate::{
  Settings,
  error::Context,
  utils::{self, fs_utils},
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
      .map_err(Into::into)
      .context("Failed to setup custom handlebar template")?;
  } else {
    handlebars
      .register_template_string("main.desktop", include_str!("./main.desktop"))
      .map_err(Into::into)
      .context("Failed to setup default handlebar template")?;
  }

  #[derive(Serialize)]
  struct DesktopTemplateParams<'a> {
    categories: &'a str,
    comment: Option<&'a str>,
    exec: &'a str,
    icon: &'a str,
    name: &'a str,
    mime_type: Option<String>,
    exec_arg: Option<&'a str>,
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

  let schemes: Vec<String> = settings
    .deep_link_protocols()
    .into_iter()
    .flatten()
    .flat_map(|protocol| &protocol.schemes)
    .map(|s| format!("x-scheme-handler/{s}"))
    .collect();

  // Launchers fall back to `%f` when Exec has no field code, which drops URLs
  // without a local path (deep links) and starts one process per file.
  // `%U` passes local files as paths and URLs as-is, all in one process;
  // `%F` also maps remote files to local paths for file-only apps.
  let has_file_associations = settings
    .file_associations()
    .is_some_and(|associations| !associations.is_empty());
  let exec_arg = if !schemes.is_empty() {
    Some("%U")
  } else if has_file_associations {
    Some("%F")
  } else {
    None
  };

  mime_type.extend(schemes);

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
      exec_arg,
      long_description: settings.long_description().unwrap_or_default().to_string(),
    },
    file,
  )?;

  Ok((file_path, dest_path))
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::bundle::{BundleBinary, BundleSettings, PackageSettings, SettingsBuilder};
  use tauri_utils::config::{AssociationExt, DeepLinkProtocol, FileAssociation};

  fn render(
    deep_link_protocols: Option<Vec<DeepLinkProtocol>>,
    file_associations: Option<Vec<FileAssociation>>,
  ) -> String {
    let tmp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let settings = SettingsBuilder::new()
      .project_out_directory(tmp_dir.path())
      .package_settings(PackageSettings {
        product_name: "TestApp".into(),
        version: "0.1.0".into(),
        description: "test".into(),
        homepage: None,
        authors: None,
        default_run: None,
      })
      .bundle_settings(BundleSettings {
        deep_link_protocols,
        file_associations,
        ..Default::default()
      })
      .binaries(vec![BundleBinary::new("test-app".into(), true)])
      .target("x86_64-unknown-linux-gnu".into())
      .build()
      .expect("failed to build settings");

    let (path, _) =
      generate_desktop_file(&settings, &None, tmp_dir.path()).expect("failed to generate");
    std::fs::read_to_string(path).expect("failed to read desktop file")
  }

  fn deep_link() -> Vec<DeepLinkProtocol> {
    vec![DeepLinkProtocol {
      schemes: vec!["my-app".into(), "other".into()],
      domains: Vec::new(),
      name: None,
      role: Default::default(),
    }]
  }

  fn file_association() -> Vec<FileAssociation> {
    vec![FileAssociation {
      ext: vec![AssociationExt("png".into())],
      content_types: None,
      name: None,
      description: None,
      role: Default::default(),
      mime_type: Some("image/png".into()),
      rank: Default::default(),
      exported_type: None,
      android_intent_action_filters: None,
    }]
  }

  #[test]
  fn exec_without_associations_has_no_field_code() {
    let desktop = render(None, None);
    assert!(desktop.contains("\nExec=test-app\n"), "{desktop}");
    assert!(!desktop.contains("MimeType="), "{desktop}");
  }

  #[test]
  fn exec_with_file_associations_takes_files() {
    let desktop = render(None, Some(file_association()));
    assert!(desktop.contains("\nExec=test-app %F\n"), "{desktop}");
    assert!(desktop.contains("\nMimeType=image/png\n"), "{desktop}");
  }

  #[test]
  fn exec_with_deep_links_takes_urls() {
    let desktop = render(Some(deep_link()), None);
    assert!(desktop.contains("\nExec=test-app %U\n"), "{desktop}");
    assert!(
      desktop.contains("\nMimeType=x-scheme-handler/my-app;x-scheme-handler/other\n"),
      "{desktop}"
    );
  }

  #[test]
  fn exec_with_deep_links_and_file_associations_takes_urls() {
    let desktop = render(Some(deep_link()), Some(file_association()));
    assert!(desktop.contains("\nExec=test-app %U\n"), "{desktop}");
    assert!(
      desktop.contains("\nMimeType=image/png;x-scheme-handler/my-app;x-scheme-handler/other\n"),
      "{desktop}"
    );
  }
}
