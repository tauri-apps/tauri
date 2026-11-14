// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

fn main() {
  // An app ACL manifest: remote pages may only call app commands a `remote` capability grants
  // (capabilities/remote.json), local pages get them from capabilities/local.json.
  tauri_build::try_build(tauri_build::Attributes::new().app_manifest(
    tauri_build::AppManifest::new().commands(&[
      "report",
      "done",
      "echo",
      "echo_bytes",
      "fail",
      "stream_numbers",
      "denied_result",
    ]),
  ))
  .expect("tauri-build failed");
}
