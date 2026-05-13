use std::sync::{Mutex, OnceLock};

pub use openharmony_ability;
pub use openharmony_ability_derive;

pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);

/// Stores the base path for OHOS app, initialized before APP is taken.
pub static BASE_PATH: OnceLock<Option<String>> = OnceLock::new();

/// Stores the module name for OHOS app, initialized before APP is taken.
pub static MODULE_NAME: OnceLock<Option<String>> = OnceLock::new();
