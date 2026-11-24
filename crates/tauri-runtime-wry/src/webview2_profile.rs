// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! WebView2 named profiles used to implement `profile_identifier` on Windows.
//!
//! See <https://learn.microsoft.com/en-us/microsoft-edge/webview2/concepts/multi-profile-support>

use std::{
  collections::BTreeSet,
  path::{Path, PathBuf},
  time::{SystemTime, UNIX_EPOCH},
};

use tauri_runtime::{Error, Result};
use webview2_com::{Microsoft::Web::WebView2::Win32::ICoreWebView2_13, take_pwstr};
use windows::core::{Interface, PWSTR};
use wry::{WebView, WebViewExtWindows};

use crate::WindowsStore;

/// Directory inside the data directory where WebView2 stores its browser data.
const BROWSER_DATA_DIR: &str = "EBWebView";
/// WebView2 stores each named profile in `EBWebView/WV2Profile_<profile name>`.
const PROFILE_DIR_PREFIX: &str = "WV2Profile_";
/// Profiles are renamed to this prefix before they're deleted.
const REMOVED_PROFILE_DIR_PREFIX: &str = "TauriRemovedProfile_";

/// Derives a WebView2 profile name from the 16-byte `profile_identifier`.
///
/// The 16 bytes are rendered as 32-char lowercase hex (valid per WebView2 profile
/// naming rules). Different identifiers always map to different profile names, and
/// the same identifier always maps to the same name.
pub fn profile_name(id: &[u8; 16]) -> String {
  id.iter().map(|b| format!("{b:02x}")).collect()
}

/// The inverse of [`profile_name`].
fn parse_profile_name(name: &str) -> Option<[u8; 16]> {
  // `from_str_radix` also accepts a leading `+`
  if name.len() != 32 || !name.bytes().all(|b| b.is_ascii_hexdigit()) {
    return None;
  }
  let mut id = [0; 16];
  for (i, byte) in id.iter_mut().enumerate() {
    *byte = u8::from_str_radix(name.get(i * 2..i * 2 + 2)?, 16).ok()?;
  }
  Some(id)
}

/// Lists the profiles created with a `profile_identifier` in the given data directories.
pub fn fetch_profile_identifiers(data_directories: &[PathBuf]) -> Vec<[u8; 16]> {
  let mut ids = BTreeSet::new();
  for data_directory in data_directories {
    let Ok(entries) = std::fs::read_dir(data_directory.join(BROWSER_DATA_DIR)) else {
      continue;
    };
    for entry in entries.flatten() {
      if !entry.file_type().is_ok_and(|t| t.is_dir()) {
        continue;
      }
      if let Some(id) = entry
        .file_name()
        .to_str()
        .and_then(|name| name.strip_prefix(PROFILE_DIR_PREFIX))
        .and_then(parse_profile_name)
      {
        ids.insert(id);
      }
    }
  }
  ids.into_iter().collect()
}

/// Removes the profile from the given data directories.
///
/// Must be called on the main thread. The files are deleted on a separate thread and `cb` is called once it's done.
pub fn remove_profile(
  windows: &WindowsStore,
  data_directories: Vec<PathBuf>,
  id: [u8; 16],
  cb: Box<dyn FnOnce(Result<()>) + Send + 'static>,
) {
  let name = profile_name(&id);

  // the browser process keeps the files of a profile in use open,
  // and the webview breaks if the profile is deleted under it
  for window in windows.0.borrow().values() {
    for webview in &window.webviews {
      if webview_profile_name(&webview.inner).is_some_and(|n| n.eq_ignore_ascii_case(&name)) {
        cb(Err(Error::ProfileInUse(webview.label.clone())));
        return;
      }
    }
  }

  // move the profile out of the way first: the rename fails as a whole if the browser process
  // still has the profile open, instead of leaving a half-deleted profile behind,
  // and it's done on the main thread so no webview can start using the profile in the meantime
  let suffix = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|d| d.as_nanos())
    .unwrap_or_default();
  let mut browser_data_dirs = Vec::new();
  for data_directory in data_directories.into_iter().collect::<BTreeSet<_>>() {
    let browser_data_dir = data_directory.join(BROWSER_DATA_DIR);
    let profile_dir = browser_data_dir.join(format!("{PROFILE_DIR_PREFIX}{name}"));
    if profile_dir.exists() {
      let removed_dir =
        browser_data_dir.join(format!("{REMOVED_PROFILE_DIR_PREFIX}{name}_{suffix}"));
      if let Err(e) = std::fs::rename(&profile_dir, &removed_dir) {
        cb(Err(Error::FailedToRemoveProfile(e)));
        return;
      }
    }
    browser_data_dirs.push(browser_data_dir);
  }

  std::thread::spawn(move || {
    for browser_data_dir in browser_data_dirs {
      // also retries profiles that could not be fully deleted previously
      remove_renamed_profiles(&browser_data_dir);
    }
    cb(Ok(()));
  });
}

fn remove_renamed_profiles(browser_data_dir: &Path) {
  let Ok(entries) = std::fs::read_dir(browser_data_dir) else {
    return;
  };
  for entry in entries.flatten() {
    if entry
      .file_name()
      .to_str()
      .is_some_and(|name| name.starts_with(REMOVED_PROFILE_DIR_PREFIX))
    {
      if let Err(e) = std::fs::remove_dir_all(entry.path()) {
        log::warn!(
          "failed to delete removed profile directory {}: {e}",
          entry.path().display()
        );
      }
    }
  }
}

fn webview_profile_name(webview: &WebView) -> Option<String> {
  unsafe {
    let profile = webview
      .webview()
      .cast::<ICoreWebView2_13>()
      .ok()?
      .Profile()
      .ok()?;
    let mut name = PWSTR::null();
    profile.ProfileName(&mut name).ok()?;
    Some(take_pwstr(name))
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const ID: [u8; 16] = [
    0x0a, 0xff, 0x12, 0x34, 0x56, 0x78, 0x9a, 0xbc, 0xde, 0xf0, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
  ];

  #[test]
  fn profile_name_is_lowercase_hex() {
    assert_eq!(profile_name(&[0u8; 16]), "00".repeat(16));
    assert_eq!(profile_name(&ID), "0aff123456789abcdef00123456789ab");
  }

  #[test]
  fn parse_profile_name_roundtrip() {
    assert_eq!(parse_profile_name(&profile_name(&ID)), Some(ID));
    assert_eq!(
      parse_profile_name("0AFF123456789ABCDEF00123456789AB"),
      Some(ID)
    );
    assert_eq!(parse_profile_name("0aff"), None);
    assert_eq!(parse_profile_name("zzff123456789abcdef00123456789ab"), None);
    assert_eq!(parse_profile_name("+aff123456789abcdef00123456789ab"), None);
  }

  #[test]
  fn fetch_profile_identifiers_lists_profile_dirs() {
    let dir = std::env::temp_dir().join(format!(
      "tauri-webview2-profile-test-{}",
      std::process::id()
    ));
    let browser_data_dir = dir.join(BROWSER_DATA_DIR);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(
      browser_data_dir.join(format!("{PROFILE_DIR_PREFIX}{}", profile_name(&ID))),
    )
    .unwrap();
    std::fs::create_dir_all(browser_data_dir.join("Default")).unwrap();
    std::fs::create_dir_all(browser_data_dir.join(format!("{PROFILE_DIR_PREFIX}custom"))).unwrap();
    std::fs::create_dir_all(browser_data_dir.join(format!(
      "{REMOVED_PROFILE_DIR_PREFIX}{}_1",
      profile_name(&[1; 16])
    )))
    .unwrap();

    let ids = fetch_profile_identifiers(&[dir.clone(), dir.join("missing")]);
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(ids, vec![ID]);
  }
}
