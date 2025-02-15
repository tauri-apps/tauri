// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! This Rust binary runs on CI and provides internal metrics results of Tauri. To learn more see [benchmark_results](https://github.com/tauri-apps/benchmark_results) repository.
//!
//! ***_Internal use only_**

#![doc(
  html_logo_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png",
  html_favicon_url = "https://github.com/tauri-apps/tauri/raw/dev/.github/icon.png"
)]

use std::{fs::File, io::BufReader};
mod utils;

fn main() {
  let tauri_data = &utils::tauri_root_path()
    .join("gh-pages")
    .join("tauri-data.json");
  let tauri_recent = &utils::tauri_root_path()
    .join("gh-pages")
    .join("tauri-recent.json");

  // current data
  let current_data_buffer = BufReader::new(
    File::open(utils::target_dir().join("bench.json")).expect("Unable to read current data file"),
  );
  let current_data: utils::BenchResult =
    serde_json::from_reader(current_data_buffer).expect("Unable to read current data buffer");

  // all data's
  let all_data_buffer =
    BufReader::new(File::open(tauri_data).expect("Unable to read all data file"));
  let mut all_data: Vec<utils::BenchResult> =
    serde_json::from_reader(all_data_buffer).expect("Unable to read all data buffer");

  // add current data to all data
  all_data.push(current_data);

  // use only latest 20 elements from all data
  let recent: Vec<utils::BenchResult> = if all_data.len() > 20 {
    all_data[all_data.len() - 20..].to_vec()
  } else {
    all_data.clone()
  };

  // write json's
  utils::write_json(
    tauri_data
      .to_str()
      .expect("Something wrong with tauri_data"),
    &serde_json::to_value(all_data).expect("Unable to build final json (all)"),
  )
  .unwrap_or_else(|_| panic!("Unable to write {:?}", tauri_data));

  utils::write_json(
    tauri_recent
      .to_str()
      .expect("Something wrong with tauri_recent"),
    &serde_json::to_value(recent).expect("Unable to build final json (recent)"),
  )
  .unwrap_or_else(|_| panic!("Unable to write {:?}", tauri_recent));
}
