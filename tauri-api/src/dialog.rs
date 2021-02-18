use std::path::{Path, PathBuf};

use rfd::FileDialog;
use tinyfiledialogs::{message_box_ok, message_box_yes_no, MessageBoxIcon, YesNo};

/// The file dialog builder.
/// Constructs file picker dialogs that can select single/multiple files or directories.
#[derive(Default)]
pub struct FileDialogBuilder(FileDialog);

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
    self.0 = self.0.set_directory(&directory);
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
pub enum AskResponse {
  /// User confirmed.
  Yes,
  /// User denied.
  No,
}

/// Displays a dialog with a message and an optional title with a "yes" and a "no" button
pub fn ask(title: impl AsRef<str>, message: impl AsRef<str>) -> AskResponse {
  match message_box_yes_no(
    title.as_ref(),
    message.as_ref(),
    MessageBoxIcon::Question,
    YesNo::No,
  ) {
    YesNo::Yes => AskResponse::Yes,
    YesNo::No => AskResponse::No,
  }
}

/// Displays a message dialog
pub fn message(title: impl AsRef<str>, message: impl AsRef<str>) {
  message_box_ok(title.as_ref(), message.as_ref(), MessageBoxIcon::Info);
}
