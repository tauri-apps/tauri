use std::path::Path;

pub use nfd::Response;
use nfd::{open_dialog, DialogType};
pub use tauri_dialog::DialogSelection;
use tauri_dialog::{DialogBuilder, DialogButtons, DialogStyle};

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

/// Displays a dialog with a message and an optional title with a "yes" and a "no" button
pub fn ask(message: impl AsRef<str>, title: impl AsRef<str>) -> DialogSelection {
  DialogBuilder::new()
    .message(message.as_ref())
    .title(title.as_ref())
    .style(DialogStyle::Question)
    .buttons(DialogButtons::YesNo)
    .build()
    .show()
}

/// Displays a message dialog
pub fn message(message: impl AsRef<str>, title: impl AsRef<str>) {
  DialogBuilder::new()
    .message(message.as_ref())
    .title(title.as_ref())
    .style(DialogStyle::Info)
    .build()
    .show();
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
