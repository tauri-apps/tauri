// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::path::Path;

fn main() {
  // `cef-dll-sys` (`links = "cef_dll_wrapper"`) downloads the CEF distribution
  // into `<cache>/<version>/cef_<os>_<arch>` and hands us that path through
  // `DEP_CEF_DLL_WRAPPER_CEF_DIR`. The `<version>` component is the download-cef
  // cache key (e.g. `150.0.10`).
  //
  // Expose it as `CEF_VERSION` so tauri-ffi can surface it: the bindings CLI
  // fetches the matching CEF for a *prebuilt* cef library, which — unlike a Rust
  // cef app whose `cef-dll-sys` build script downloads CEF at compile time — has
  // no Cargo.lock for the CLI to resolve the version from.
  println!("cargo::rerun-if-env-changed=DEP_CEF_DLL_WRAPPER_CEF_DIR");
  let version = std::env::var("DEP_CEF_DLL_WRAPPER_CEF_DIR")
    .ok()
    .and_then(|dir| {
      // `<cache>/<version>/cef_<os>_<arch>` -> `<version>`
      Path::new(&dir)
        .parent()
        .and_then(Path::file_name)
        .map(|name| name.to_string_lossy().into_owned())
    })
    .unwrap_or_default();
  println!("cargo::rustc-env=TAURI_RUNTIME_CEF_VERSION={version}");
}
