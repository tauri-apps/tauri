// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

fn main() {
  use std::{
    path::PathBuf,
    process::{Command, Stdio},
  };

  let video_file = PathBuf::from("test_video.mp4");
  let video_url =
    "http://commondatastorage.googleapis.com/gtv-videos-bucket/sample/BigBuckBunny.mp4";

  if !video_file.exists() {
    // Downloading with curl this saves us from adding
    // a Rust HTTP client dependency.
    println!("Downloading {}", video_url);
    let status = Command::new("curl")
      .arg("-L")
      .arg("-o")
      .arg(&video_file)
      .arg(video_url)
      .stdout(Stdio::inherit())
      .stderr(Stdio::inherit())
      .output()
      .unwrap();

    assert!(status.status.success());
    assert!(video_file.exists());
  }

  tauri::Builder::default()
    .run(tauri::generate_context!(
      "../../examples/streaming/tauri.conf.json"
    ))
    .expect("error while running tauri application");
}
