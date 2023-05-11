pub use tauri_utils::{assets::EmbeddedAssets, Context};

pub fn context() -> Context<EmbeddedAssets> {
  tauri::build_script_context!()
}
