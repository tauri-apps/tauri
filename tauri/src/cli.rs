use once_cell::sync::OnceCell;
use tauri_api::cli::Matches;

static MATCHES: OnceCell<Matches> = OnceCell::new();

pub(crate) fn set_matches(matches: Matches) -> crate::Result<()> {
  MATCHES
    .set(matches)
    .map_err(|_| anyhow::anyhow!("failed to set once_cell matches"))
}

pub fn get_matches() -> Option<&'static Matches> {
  MATCHES.get()
}
