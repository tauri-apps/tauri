// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Downloads: the webview download handler (destination from the settings), a notification
//! when one finishes (notification plugin), the taskbar badge count, progress bar and attention
//! request, and `qtbrowser://downloads`, which gets live updates through a channel. It shows
//! image thumbnails through the asset protocol and opens files with the opener plugin.

use std::{
  path::PathBuf,
  time::{SystemTime, UNIX_EPOCH},
};

use serde::Serialize;
use serde_json::json;
use tauri::{
  AppHandle, Emitter, EventTarget, Manager, Runtime, UserAttentionType, Webview,
  ipc::Channel,
  webview::DownloadEvent,
  window::{ProgressBarState, ProgressBarStatus},
};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_opener::OpenerExt;

use crate::browser::{browser, chrome_label};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Download {
  pub id: u32,
  pub url: String,
  pub path: PathBuf,
  /// `downloading`, `completed` or `failed`.
  pub state: &'static str,
  /// Milliseconds since the Unix epoch.
  pub started: u64,
}

#[derive(Default)]
pub struct Downloads {
  items: Vec<Download>,
  next: u32,
  /// `qtbrowser://downloads` pages listening for changes.
  watchers: Vec<Channel<serde_json::Value>>,
}

impl Downloads {
  /// How many downloads are running.
  pub fn active(&self) -> usize {
    self
      .items
      .iter()
      .filter(|d| d.state == "downloading")
      .count()
  }
}

fn now() -> u64 {
  SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map_or(0, |d| d.as_millis() as u64)
}

/// A short notice in a window's title bar.
pub fn toast<R: Runtime>(app: &AppHandle<R>, window: &str, message: &str) {
  let _ = app.emit_to(
    EventTarget::webview(chrome_label(window)),
    "browser-toast",
    message,
  );
}

/// The download handler of every tab.
pub fn on_download<R: Runtime>(
  app: &AppHandle<R>,
  webview: &Webview<R>,
  event: DownloadEvent<'_>,
) -> bool {
  let window = webview.window();
  match event {
    DownloadEvent::Requested { url, destination } => {
      // A download folder from the settings replaces the default one.
      let dir = browser(app).settings.download_dir.clone();
      if !dir.trim().is_empty()
        && let Some(name) = destination.file_name().map(ToOwned::to_owned)
      {
        *destination = PathBuf::from(dir).join(name);
      }
      let path = destination.clone();
      {
        let mut inner = browser(app);
        let d = &mut inner.downloads;
        d.next += 1;
        let id = d.next;
        d.items.insert(
          0,
          Download {
            id,
            url: url.to_string(),
            path: path.clone(),
            state: "downloading",
            started: now(),
          },
        );
      }
      log::info!("downloading {url} to {}", path.display());
      toast(
        app,
        window.label(),
        &format!("Downloading {}", file_name(&path)),
      );
    }
    DownloadEvent::Finished { url, path, success } => {
      let item = {
        let mut inner = browser(app);
        let item = inner
          .downloads
          .items
          .iter_mut()
          .find(|d| d.state == "downloading" && d.url == url.as_str());
        item.map(|d| {
          d.state = if success { "completed" } else { "failed" };
          if let Some(p) = path {
            d.path = p;
          }
          d.clone()
        })
      };
      if let Some(d) = item {
        let name = file_name(&d.path);
        let (title, message) = if success {
          ("Download complete", format!("Downloaded {name}"))
        } else {
          ("Download failed", format!("Couldn't download {name}"))
        };
        toast(app, window.label(), &message);
        if browser(app).settings.notifications {
          let _ = app.notification().builder().title(title).body(&name).show();
        }
        // Flash the taskbar entry if the user is elsewhere.
        if !window.is_focused().unwrap_or(true) {
          let _ = window.request_user_attention(Some(UserAttentionType::Informational));
        }
      }
    }
    _ => {}
  }
  changed(app);
  true
}

fn file_name(path: &std::path::Path) -> String {
  path.file_name().map_or_else(
    || path.display().to_string(),
    |n| n.to_string_lossy().into_owned(),
  )
}

/// The list changed: update the downloads pages, the taskbar badge and progress bar, and the
/// download count in the title bars.
fn changed<R: Runtime>(app: &AppHandle<R>) {
  let (list, active, windows) = {
    let inner = browser(app);
    (
      json!(inner.downloads.items),
      inner.downloads.active(),
      inner.windows.keys().cloned().collect::<Vec<_>>(),
    )
  };
  // Sent without the lock held; pages that are gone (failed sends) are dropped.
  let mut watchers = std::mem::take(&mut browser(app).downloads.watchers);
  watchers.retain(|c| c.send(list.clone()).is_ok());
  browser(app).downloads.watchers.extend(watchers);
  for label in windows {
    if let Some(w) = app.get_window(&label) {
      let _ = w.set_badge_count((active > 0).then_some(active as i64));
      let _ = w.set_progress_bar(ProgressBarState {
        status: Some(if active > 0 {
          ProgressBarStatus::Indeterminate
        } else {
          ProgressBarStatus::None
        }),
        progress: None,
      });
    }
    crate::browser::refresh(app, &label);
  }
}

/// Let the asset protocol (download thumbnails) read the configured download folder; the
/// system's downloads folder is allowed by `tauri.conf.json`.
pub fn allow_download_dir<R: Runtime>(app: &AppHandle<R>) {
  let dir = browser(app).settings.download_dir.clone();
  if !dir.trim().is_empty() {
    let _ = app.asset_protocol_scope().allow_directory(&dir, true);
  }
}

// --- commands -------------------------------------------------------------------------------------

/// `qtbrowser://downloads` subscribes: it gets the list now and after every change.
#[tauri::command]
pub fn watch_downloads<R: Runtime>(app: AppHandle<R>, channel: Channel<serde_json::Value>) {
  let mut inner = browser(&app);
  let _ = channel.send(json!(inner.downloads.items));
  inner.downloads.watchers.push(channel);
}

/// `open` (the file), `show` (in its folder), `remove` (from the list) or `clear` (finished ones).
#[tauri::command]
pub fn download_action<R: Runtime>(
  app: AppHandle<R>,
  id: Option<u32>,
  action: String,
) -> Result<(), String> {
  let path = id.and_then(|id| {
    browser(&app)
      .downloads
      .items
      .iter()
      .find(|d| d.id == id)
      .map(|d| d.path.clone())
  });
  match (action.as_str(), path) {
    ("open", Some(p)) => app
      .opener()
      .open_path(p.to_string_lossy(), None::<&str>)
      .map_err(|e| e.to_string())?,
    ("show", Some(p)) => app
      .opener()
      .reveal_item_in_dir(p)
      .map_err(|e| e.to_string())?,
    ("remove", _) => browser(&app).downloads.items.retain(|d| Some(d.id) != id),
    ("clear", _) => browser(&app)
      .downloads
      .items
      .retain(|d| d.state == "downloading"),
    _ => return Err(format!("unknown download action {action}")),
  }
  changed(&app);
  Ok(())
}
