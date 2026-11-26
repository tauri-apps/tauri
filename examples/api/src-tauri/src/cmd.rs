// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::{Deserialize, Serialize};
use std::{
  collections::BTreeMap,
  fs, io,
  path::{Path, PathBuf},
};
use tauri::{
  AppHandle, Manager, Runtime, command,
  ipc::{Channel, CommandScope},
};

#[derive(Debug, Deserialize)]
#[allow(unused)]
pub struct RequestBody {
  id: i32,
  name: String,
}

#[derive(Debug, Deserialize)]
pub struct LogScope {
  event: String,
}

#[command]
pub fn log_operation(
  event: String,
  payload: Option<String>,
  command_scope: CommandScope<LogScope>,
) -> Result<(), &'static str> {
  if command_scope.denies().iter().any(|s| s.event == event) {
    Err("denied")
  } else if !command_scope.allows().iter().any(|s| s.event == event) {
    Err("not allowed")
  } else {
    log::info!("{event} {payload:?}");
    Ok(())
  }
}

#[derive(Serialize)]
pub struct ApiResponse {
  message: String,
}

#[command]
pub fn perform_request(endpoint: String, body: RequestBody) -> ApiResponse {
  println!("{endpoint} {body:?}");
  ApiResponse {
    message: "message response".into(),
  }
}

#[command]
pub fn echo(request: tauri::ipc::Request<'_>) -> tauri::ipc::Response {
  tauri::ipc::Response::new(request.body().clone())
}

#[command]
pub fn spam(channel: Channel<i32>) -> tauri::Result<()> {
  for i in 1..=1_000 {
    channel.send(i)?;
  }
  Ok(())
}

/// Directory inside the app cache directory the Asset view's media files are
/// downloaded to. The asset protocol scope in `tauri.conf.json` allows it.
const ASSET_MEDIA_DIR: &str = "asset-view";

/// CC0 media files from the MDN interactive examples.
const ASSET_MEDIA_FILES: &[(&str, &str)] = &[
  (
    "image.jpg",
    "https://interactive-examples.mdn.mozilla.net/media/cc0-images/grapefruit-slice-332-332.jpg",
  ),
  (
    "video.mp4",
    "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.mp4",
  ),
  (
    "video.webm",
    "https://interactive-examples.mdn.mozilla.net/media/cc0-videos/flower.webm",
  ),
  (
    "audio.mp3",
    "https://interactive-examples.mdn.mozilla.net/media/cc0-audio/t-rex-roar.mp3",
  ),
];

#[derive(Serialize)]
pub struct AssetMedia {
  /// The media files by file name, `None` when they are not all downloaded yet.
  files: Option<BTreeMap<&'static str, PathBuf>>,
  /// A path the asset protocol scope does not allow.
  out_of_scope: PathBuf,
}

/// Returns the media files used by the Asset view, downloading the missing
/// ones first when `download` is set.
#[command]
pub async fn asset_media<R: Runtime>(
  app: AppHandle<R>,
  download: bool,
) -> Result<AssetMedia, String> {
  let cache_dir = app.path().app_cache_dir().map_err(|e| e.to_string())?;
  let media_dir = cache_dir.join(ASSET_MEDIA_DIR);

  let files = tauri::async_runtime::spawn_blocking(move || {
    let mut files = BTreeMap::new();
    for (name, url) in ASSET_MEDIA_FILES {
      let path = media_dir.join(name);
      if !path.exists() {
        if !download {
          return Ok(None);
        }
        download_file(url, &path).map_err(|e| format!("failed to download {url}: {e}"))?;
      }
      files.insert(*name, path);
    }
    Ok::<_, String>(Some(files))
  })
  .await
  .map_err(|e| e.to_string())??;

  Ok(AssetMedia {
    files,
    out_of_scope: cache_dir.join("not-in-the-asset-scope.txt"),
  })
}

fn download_file(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent)?;
  }

  // download to a temporary file so an interrupted download is not mistaken for the media file
  let partial = path.with_extension("part");
  let mut response = ureq::get(url).call()?;
  io::copy(
    &mut response.body_mut().as_reader(),
    &mut fs::File::create(&partial)?,
  )?;
  fs::rename(partial, path)?;

  Ok(())
}
