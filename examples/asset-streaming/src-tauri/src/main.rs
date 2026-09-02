// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Plays a video over the `asset://` protocol.
//!
//! WebKitGTK hands `<video>` sources to GStreamer instead of the custom
//! protocol handler, so this only plays when a GStreamer plugin providing an
//! `asset://` URI handler is reachable.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{path::PathBuf, process::Command};

const VIDEO_FILE: &str = "streaming_example_test_video.mp4";
const VIDEO_URL: &str =
  "https://test-videos.co.uk/vids/bigbuckbunny/mp4/h264/720/Big_Buck_Bunny_720_10s_1MB.mp4";

fn video_file() -> PathBuf {
  std::env::temp_dir().join(VIDEO_FILE)
}

#[tauri::command]
fn video_path() -> String {
  video_file().to_string_lossy().into_owned()
}

fn main() {
  let path = video_file();
  if !path.exists() {
    println!("Downloading {VIDEO_URL} to {}", path.display());
    let status = Command::new("curl")
      .args(["-L", "-o"])
      .arg(&path)
      .arg(VIDEO_URL)
      .status()
      .expect("failed to run curl");
    assert!(status.success(), "failed to download the video");
  }

  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![video_path])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
