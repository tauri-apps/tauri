use once_cell::sync::Lazy;
use tauri_api::cli::Matches;

/// Gets the CLI arg matches.
pub fn get_matches() -> &'static Option<Matches> {
  static MATCHES: Lazy<Option<Matches>> = Lazy::new(|| tauri_api::cli::get_matches().ok());

  &MATCHES
}
