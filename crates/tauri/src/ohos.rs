use std::sync::Mutex;

pub use openharmony_ability;
pub use openharmony_ability_derive;

pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);

// Re-exported for the mobile entry-point macro in the application crate.
#[doc(hidden)]
pub use napi_derive_ohos;
#[doc(hidden)]
pub use napi_ohos;

/// Latest device fold state, or `None` before initialization/when unavailable.
pub fn fold_state() -> Option<crate::FoldState> {
  crate::app::fold::native::snapshot()
}

/// Called only by the generated native entry point, never by webview IPC.
#[doc(hidden)]
pub fn update_fold_state(is_foldable: Option<bool>, status: Option<u32>) {
  crate::app::fold::native::update(
    is_foldable.map(|foldable| crate::FoldState::from_native(foldable, status.unwrap_or(0))),
  );
}
