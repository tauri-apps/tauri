use std::sync::Mutex;

pub use openharmony_ability;
pub use openharmony_ability_derive;

pub static APP: Mutex<Option<openharmony_ability::OpenHarmonyApp>> = Mutex::new(None);
