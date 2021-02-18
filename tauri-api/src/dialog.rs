use std::path::Path;

pub use nfd::Response;
use nfd::{open_dialog, DialogType};

use tinyfiledialogs::{message_box_ok, message_box_yes_no, MessageBoxIcon, YesNo};

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

fn open_dialog_internal(
  dialog_type: DialogType,
  filter: Option<impl AsRef<str>>,
  default_path: Option<impl AsRef<Path>>,
) -> crate::Result<Response> {
  let response = open_dialog(
    filter.map(|s| s.as_ref().to_string()).as_deref(),
    default_path
      .map(|s| s.as_ref().to_string_lossy().to_string())
      .as_deref(),
    dialog_type,
  )
  .map_err(|e| crate::Error::Dialog(e.to_string()))?;
  match response {
    Response::Cancel => Err(crate::Error::DialogCancelled),
    _ => Ok(response),
  }
}

/// Open single select file dialog
pub fn select(
  filter_list: Option<impl AsRef<str>>,
  default_path: Option<impl AsRef<Path>>,
) -> crate::Result<Response> {
  open_dialog_internal(DialogType::SingleFile, filter_list, default_path)
}

/// Open multiple select file dialog
pub fn select_multiple(
  filter_list: Option<impl AsRef<str>>,
  default_path: Option<impl AsRef<Path>>,
) -> crate::Result<Response> {
  open_dialog_internal(DialogType::MultipleFiles, filter_list, default_path)
}

/// Open save dialog
pub fn save_file(
  filter_list: Option<impl AsRef<str>>,
  default_path: Option<impl AsRef<Path>>,
) -> crate::Result<Response> {
  open_dialog_internal(DialogType::SaveFile, filter_list, default_path)
}

/// Open pick folder dialog
pub fn pick_folder(default_path: Option<impl AsRef<Path>>) -> crate::Result<Response> {
  let filter: Option<String> = None;
  open_dialog_internal(DialogType::PickFolder, filter, default_path)
}
