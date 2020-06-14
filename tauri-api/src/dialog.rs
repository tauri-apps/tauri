pub use nfd::Response;
use nfd::{open_dialog, DialogType};
pub use tauri_dialog::DialogSelection;
use tauri_dialog::{DialogBuilder, DialogButtons, DialogStyle};

fn open_dialog_internal(
  dialog_type: DialogType,
  filter: Option<String>,
  default_path: Option<String>,
) -> crate::Result<Response> {
  let response = open_dialog(filter.as_deref(), default_path.as_deref(), dialog_type)?;
  match response {
    Response::Cancel => Err(crate::Error::Dialog("user cancelled".into()).into()),
    _ => Ok(response),
  }
}

/// Displays a dialog with a message and an optional title with a "yes" and a "no" button.
pub fn ask<'a>(message: &'a str, title: &'a str) -> DialogSelection {
  DialogBuilder::new()
    .message(message)
    .title(title)
    .style(DialogStyle::Question)
    .buttons(DialogButtons::YesNo)
    .build()
    .show()
}

/// Open single select file dialog
pub fn select(
  filter_list: Option<String>,
  default_path: Option<String>,
) -> crate::Result<Response> {
  open_dialog_internal(DialogType::SingleFile, filter_list, default_path)
}

/// Open mulitple select file dialog
pub fn select_multiple(
  filter_list: Option<String>,
  default_path: Option<String>,
) -> crate::Result<Response> {
  open_dialog_internal(DialogType::MultipleFiles, filter_list, default_path)
}

/// Open save dialog
pub fn save_file(
  filter_list: Option<String>,
  default_path: Option<String>,
) -> crate::Result<Response> {
  open_dialog_internal(DialogType::SaveFile, filter_list, default_path)
}

/// Open pick folder dialog
pub fn pick_folder(default_path: Option<String>) -> crate::Result<Response> {
  open_dialog_internal(DialogType::PickFolder, None, default_path)
}
