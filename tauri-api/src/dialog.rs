use nfd::{DialogType, open_dialog};
pub use nfd::Response;

fn open_dialog_internal(dialog_type: DialogType, filter: Option<String>, default_path: Option<String>) -> crate::Result<Response> {
  open_dialog(filter.as_deref(), default_path.as_deref(), dialog_type)
    .map_err(|err| crate::Error::with_chain(err, "open dialog failed"))
    .and_then(|response| {
      match response {
        Response::Cancel => Err(crate::Error::from("user cancelled")),
        _ => Ok(response)
      }
    })
}

/// Open single select file dialog
pub fn select(filter_list: Option<String>, default_path: Option<String>) -> crate::Result<Response> {
    open_dialog_internal(DialogType::SingleFile, filter_list, default_path)
}

/// Open mulitple select file dialog
pub fn select_multiple(filter_list: Option<String>, default_path: Option<String>) -> crate::Result<Response> {
    open_dialog_internal(DialogType::MultipleFiles, filter_list, default_path)
}

/// Open save dialog
pub fn save_file(filter_list: Option<String>, default_path: Option<String>) -> crate::Result<Response> {
    open_dialog_internal(DialogType::SaveFile, filter_list, default_path)
}

/// Open pick folder dialog
pub fn pick_folder(default_path: Option<String>) -> crate::Result<Response> {
    open_dialog_internal(DialogType::PickFolder, None, default_path)
}
