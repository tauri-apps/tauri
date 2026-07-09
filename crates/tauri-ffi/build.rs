// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Collects build-time data that `tauri-ffi` embeds so that FFI consumers
//! never need Rust codegen (`generate_context!` / `tauri-build`):
//!
//! 1. ACL manifests — the `tauri` crate's build script defines core plugin
//!    permissions and advertises them through `links` metadata
//!    (`DEP_TAURI_*_PERMISSION_FILES_PATH`). We collect them exactly the way
//!    `tauri-build` does for apps and embed them as JSON, so capabilities can
//!    be resolved at runtime (`dynamic-acl`).
//! 2. Global API scripts — the script behind `window.__TAURI__`
//!    (`DEP_TAURI_GLOBAL_API_SCRIPT_PATH`), embedded via `include_str!` and
//!    injected when the config enables `app.withGlobalTauri`.
//!
//! Compiled-in plugins added to this crate later will surface through the
//! same two channels automatically.

use std::{collections::BTreeMap, env, fs, path::PathBuf};

fn main() {
  let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR not set"));

  // ---- ABI version from the manifest ---------------------------------------
  // api-manifest.json is the single source of truth for the ABI; the exported
  // tauri_ffi_abi_version() must always match what the generators emitted.
  println!("cargo:rerun-if-changed=api-manifest.json");
  let manifest: serde_json::Value =
    serde_json::from_str(&fs::read_to_string("api-manifest.json").expect("missing api-manifest.json"))
      .expect("invalid api-manifest.json");
  let abi_version = manifest["abiVersion"]
    .as_u64()
    .expect("api-manifest.json: abiVersion must be a number");
  fs::write(
    out_dir.join("abi_version.rs"),
    format!("const ABI_VERSION: u32 = {abi_version};\n"),
  )
  .expect("failed to write abi_version.rs");

  // ---- ACL manifests -------------------------------------------------------
  let permissions =
    tauri_utils::acl::build::read_permissions().expect("failed to read permission metadata");
  let mut scope_schemas = tauri_utils::acl::build::read_global_scope_schemas()
    .expect("failed to read global scope schemas");

  let manifests: BTreeMap<String, tauri_utils::acl::manifest::Manifest> = permissions
    .into_iter()
    .map(|(name, files)| {
      let schema = scope_schemas.remove(&name);
      (name, tauri_utils::acl::manifest::Manifest::new(files, schema))
    })
    .collect();

  fs::write(
    out_dir.join("acl-manifests.json"),
    serde_json::to_string(&manifests).expect("failed to serialize ACL manifests"),
  )
  .expect("failed to write acl-manifests.json");

  // ---- global API scripts (window.__TAURI__) -------------------------------
  let mut scripts: Vec<String> = env::vars()
    .filter(|(k, _)| k.starts_with("DEP_") && k.ends_with("_GLOBAL_API_SCRIPT_PATH"))
    .map(|(_, path)| path)
    .collect();
  scripts.sort();

  let entries: String = scripts
    .iter()
    .map(|path| format!("  include_str!({path:?}),\n"))
    .collect();
  fs::write(
    out_dir.join("global_api_scripts.rs"),
    format!("static GLOBAL_API_SCRIPTS: &[&str] = &[\n{entries}];\n"),
  )
  .expect("failed to write global_api_scripts.rs");
}
