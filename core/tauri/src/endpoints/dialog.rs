// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{InvokeContext, InvokeResponse};
use crate::Runtime;
#[cfg(any(dialog_open, dialog_save))]
use crate::{api::dialog::blocking::FileDialogBuilder, Manager, Scopes};
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

use std::path::PathBuf;

#[allow(dead_code)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogFilter {
  name: String,
  extensions: Vec<String>,
}

/// The options for the open dialog API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenDialogOptions {
  /// The title of the dialog window.
  pub title: Option<String>,
  /// The filters of the dialog.
  #[serde(default)]
  pub filters: Vec<DialogFilter>,
  /// Whether the dialog allows multiple selection or not.
  #[serde(default)]
  pub multiple: bool,
  /// Whether the dialog is a directory selection (`true` value) or file selection (`false` value).
  #[serde(default)]
  pub directory: bool,
  /// The initial path of the dialog.
  pub default_path: Option<PathBuf>,
  /// If [`Self::directory`] is true, indicates that it will be read recursively later.
  /// Defines whether subdirectories will be allowed on the scope or not.
  #[serde(default)]
  pub recursive: bool,
}

/// The options for the save dialog API.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveDialogOptions {
  /// The title of the dialog window.
  pub title: Option<String>,
  /// The filters of the dialog.
  #[serde(default)]
  pub filters: Vec<DialogFilter>,
  /// The initial path of the dialog.
  pub default_path: Option<PathBuf>,
}

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
#[allow(clippy::enum_variant_names)]
pub enum Cmd {
  /// The open dialog API.
  OpenDialog {
    options: OpenDialogOptions,
  },
  /// The save dialog API.
  SaveDialog {
    options: SaveDialogOptions,
  },
  MessageDialog {
    message: String,
  },
  AskDialog {
    title: Option<String>,
    message: String,
  },
  ConfirmDialog {
    title: Option<String>,
    message: String,
  },
}

impl Cmd {
  #[module_command_handler(dialog_open, "dialog > open")]
  #[allow(unused_variables)]
  fn open_dialog<R: Runtime>(
    context: InvokeContext<R>,
    options: OpenDialogOptions,
  ) -> super::Result<InvokeResponse> {
    let mut dialog_builder = FileDialogBuilder::new();
    #[cfg(any(windows, target_os = "macos"))]
    {
      dialog_builder = dialog_builder.set_parent(&context.window);
    }
    if let Some(default_path) = options.default_path {
      dialog_builder = set_default_path(dialog_builder, default_path);
    }
    for filter in options.filters {
      let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
      dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
    }

    let scopes = context.window.state::<Scopes>();

    let res = if options.directory {
      let folder = dialog_builder.pick_folder();
      if let Some(path) = &folder {
        scopes
          .allow_directory(path, options.recursive)
          .map_err(crate::error::into_anyhow)?;
      }
      folder.into()
    } else if options.multiple {
      let files = dialog_builder.pick_files();
      if let Some(files) = &files {
        for file in files {
          scopes.allow_file(file).map_err(crate::error::into_anyhow)?;
        }
      }
      files.into()
    } else {
      let file = dialog_builder.pick_file();
      if let Some(file) = &file {
        scopes.allow_file(file).map_err(crate::error::into_anyhow)?;
      }
      file.into()
    };

    Ok(res)
  }

  #[module_command_handler(dialog_save, "dialog > save")]
  #[allow(unused_variables)]
  fn save_dialog<R: Runtime>(
    context: InvokeContext<R>,
    options: SaveDialogOptions,
  ) -> super::Result<Option<PathBuf>> {
    let mut dialog_builder = FileDialogBuilder::new();
    #[cfg(any(windows, target_os = "macos"))]
    {
      dialog_builder = dialog_builder.set_parent(&context.window);
    }
    if let Some(default_path) = options.default_path {
      dialog_builder = set_default_path(dialog_builder, default_path);
    }
    for filter in options.filters {
      let extensions: Vec<&str> = filter.extensions.iter().map(|s| &**s).collect();
      dialog_builder = dialog_builder.add_filter(filter.name, &extensions);
    }

    let scopes = context.window.state::<Scopes>();

    let path = dialog_builder.save_file();
    if let Some(p) = &path {
      scopes.allow_file(p).map_err(crate::error::into_anyhow)?;
    }

    Ok(path)
  }

  #[module_command_handler(dialog_message, "dialog > message")]
  fn message_dialog<R: Runtime>(context: InvokeContext<R>, message: String) -> super::Result<()> {
    crate::api::dialog::blocking::message(
      Some(&context.window),
      &context.window.app_handle.package_info().name,
      message,
    );
    Ok(())
  }

  #[module_command_handler(dialog_ask, "dialog > ask")]
  fn ask_dialog<R: Runtime>(
    context: InvokeContext<R>,
    title: Option<String>,
    message: String,
  ) -> super::Result<bool> {
    Ok(crate::api::dialog::blocking::ask(
      Some(&context.window),
      title.unwrap_or_else(|| context.window.app_handle.package_info().name.clone()),
      message,
    ))
  }

  #[module_command_handler(dialog_confirm, "dialog > confirm")]
  fn confirm_dialog<R: Runtime>(
    context: InvokeContext<R>,
    title: Option<String>,
    message: String,
  ) -> super::Result<bool> {
    Ok(crate::api::dialog::blocking::confirm(
      Some(&context.window),
      title.unwrap_or_else(|| context.window.app_handle.package_info().name.clone()),
      message,
    ))
  }
}

#[cfg(any(dialog_open, dialog_save))]
fn set_default_path(
  mut dialog_builder: FileDialogBuilder,
  default_path: PathBuf,
) -> FileDialogBuilder {
  if default_path.is_file() || !default_path.exists() {
    if let (Some(parent), Some(file_name)) = (default_path.parent(), default_path.file_name()) {
      dialog_builder = dialog_builder.set_directory(parent);
      dialog_builder = dialog_builder.set_file_name(&file_name.to_string_lossy().to_string());
    } else {
      dialog_builder = dialog_builder.set_directory(default_path);
    }
    dialog_builder
  } else {
    dialog_builder.set_directory(default_path)
  }
}

#[cfg(test)]
mod tests {
  use super::{OpenDialogOptions, SaveDialogOptions};
  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for OpenDialogOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        filters: Vec::new(),
        multiple: bool::arbitrary(g),
        directory: bool::arbitrary(g),
        default_path: Option::arbitrary(g),
        title: Option::arbitrary(g),
        recursive: bool::arbitrary(g),
      }
    }
  }

  impl Arbitrary for SaveDialogOptions {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        filters: Vec::new(),
        default_path: Option::arbitrary(g),
        title: Option::arbitrary(g),
      }
    }
  }

  #[tauri_macros::module_command_test(dialog_open, "dialog > open")]
  #[quickcheck_macros::quickcheck]
  fn open_dialog(_options: OpenDialogOptions) {}

  #[tauri_macros::module_command_test(dialog_save, "dialog > save")]
  #[quickcheck_macros::quickcheck]
  fn save_dialog(_options: SaveDialogOptions) {}
}
