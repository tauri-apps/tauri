// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{fs::File, io::BufReader};
mod utils;

fn main() {
  let platform = if cfg!(target_os = "macos") {
    "macos"
  } else if cfg!(target_os = "windows") {
    "windows"
  } else {
    "linux"
  };
  let wry_data = &utils::wry_root_path()
    .join("gh-pages")
    .join(format!("wry-data-{platform}.json"));
  let wry_recent = &utils::wry_root_path()
    .join("gh-pages")
    .join(format!("wry-recent-{platform}.json"));

  // current data
  let current_data_buffer = BufReader::new(
    File::open(utils::target_dir().join("bench.json")).expect("Unable to read current data file"),
  );
  let current_data: utils::BenchResult =
    serde_json::from_reader(current_data_buffer).expect("Unable to read current data buffer");

  // all data's
  let all_data_buffer = BufReader::new(File::open(wry_data).expect("Unable to read all data file"));
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

  // write jsons
  utils::write_json(
    wry_data,
    &serde_json::to_value(&all_data).expect("Unable to build final json (all)"),
  )
  .unwrap_or_else(|_| panic!("Unable to write {}", wry_data.display()));

  utils::write_json(
    wry_recent,
    &serde_json::to_value(recent).expect("Unable to build final json (recent)"),
  )
  .unwrap_or_else(|_| panic!("Unable to write {}", wry_recent.display()));
}
