// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(any(dialog_open, dialog_save))]
use std::path::{Path, PathBuf};

/// The file dialog builder.
/// Constructs file picker dialogs that can select single/multiple files or directories.
#[cfg(any(dialog_open, dialog_save))]
#[derive(Default)]
pub struct FileDialogBuilder(rfd::FileDialog);

#[cfg(any(dialog_open, dialog_save))]
impl FileDialogBuilder {
  /// Gets the default file dialog builder.
  pub fn new() -> Self {
    Default::default()
  }

  /// Add file extension filter. Takes in the name of the filter, and list of extensions
  pub fn add_filter(mut self, name: impl AsRef<str>, extensions: &[&str]) -> Self {
    self.0 = self.0.add_filter(name.as_ref(), extensions);
    self
  }

  /// Set starting directory of the dialog.
  pub fn set_directory<P: AsRef<Path>>(mut self, directory: P) -> Self {
    self.0 = self.0.set_directory(directory);
    self
  }

  /// Set starting file name of the dialog.
  pub fn set_file_name(mut self, file_name: &str) -> Self {
    self.0 = self.0.set_file_name(file_name);
    self
  }

  #[cfg(windows)]
  /// Sets the parent window of the dialog.
  pub fn set_parent<W: raw_window_handle::HasRawWindowHandle>(mut self, parent: &W) -> Self {
    self.0 = self.0.set_parent(parent);
    self
  }

  /// Pick one file.
  pub fn pick_file(self) -> Option<PathBuf> {
    self.0.pick_file()
  }

  /// Pick multiple files.
  pub fn pick_files(self) -> Option<Vec<PathBuf>> {
    self.0.pick_files()
  }

  /// Pick one folder.
  pub fn pick_folder(self) -> Option<PathBuf> {
    self.0.pick_folder()
  }

  /// Opens save file dialog.
  pub fn save_file(self) -> Option<PathBuf> {
    self.0.save_file()
  }
}

/// Response for the ask dialog
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AskResponse {
  /// User confirmed.
  Yes,
  /// User denied.
  No,
}

/// Displays a dialog with a message and an optional title with a "yes" and a "no" button
pub fn ask(title: impl AsRef<str>, message: impl AsRef<str>) -> AskResponse {
  match rfd::MessageDialog::new()
    .set_title(title.as_ref())
    .set_description(message.as_ref())
    .set_buttons(rfd::MessageButtons::YesNo)
    .set_level(rfd::MessageLevel::Info)
    .show()
  {
    true => AskResponse::Yes,
    false => AskResponse::No,
  }
}

/// Displays a message dialog
pub fn message(title: impl AsRef<str>, message: impl AsRef<str>) {
  rfd::MessageDialog::new()
    .set_title(title.as_ref())
    .set_description(message.as_ref())
    .set_buttons(rfd::MessageButtons::Ok)
    .set_level(rfd::MessageLevel::Info)
    .show();
}
