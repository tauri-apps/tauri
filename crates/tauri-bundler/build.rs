// Copyright 2019-2025 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  env, fs,
  path::{Path, PathBuf},
  process::Command,
};

fn main() {
  let target = env::var("TARGET").unwrap_or_default();
  let host = env::var("HOST").unwrap_or_default();

  // Only build/embed the CEF helper when compiling `tauri-bundler` for macOS.
  if !target.contains("apple-darwin") {
    return;
  }

  // We need `lipo` and a functioning macOS toolchain to produce a universal Mach-O.
  if !host.contains("apple-darwin") {
    panic!(
      "Building tauri-bundler for macOS requires a macOS host to build/embed the CEF helper binary"
    );
  }

  let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR not set"));
  let bundler_manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

  let helper_root = bundler_manifest_dir
    .parent() // crates/
    .and_then(|p| p.parent()) // repo root
    .map(|p| p.join("cef-helper"))
    .expect("failed to compute cef-helper path");

  let helper_manifest = helper_root.join("Cargo.toml");
  let helper_src = helper_root.join("src");

  // Rebuild whenever the helper crate's manifest or any source file changes.
  println!("cargo:rerun-if-changed={}", helper_manifest.display());
  println!("cargo:rerun-if-changed={}", helper_src.display());

  // Copy the helper crate sources into OUT_DIR so generated files (Cargo.lock,
  // target dir) stay out of the repo checkout. Walk `src/` so additional
  // modules (e.g. cef-helper/src/notification.rs) ride along with main.rs.
  let helper_src_dir = out_dir.join("cef-helper-src");
  let helper_src_manifest = helper_src_dir.join("Cargo.toml");
  let helper_src_src = helper_src_dir.join("src");
  fs::create_dir_all(&helper_src_src).expect("failed to create cef-helper-src directory");
  fs::copy(&helper_manifest, &helper_src_manifest).expect("failed to copy cef-helper Cargo.toml");
  copy_src_tree(&helper_src, &helper_src_src);

  let cargo = env::var("CARGO").unwrap_or_else(|_| "cargo".into());

  let helper_target_dir = out_dir.join("cef-helper-target");
  let aarch64 = build_helper(
    &cargo,
    &helper_src_manifest,
    &helper_target_dir,
    "aarch64-apple-darwin",
    "tauri-cef-helper",
  );
  let x86_64 = build_helper(
    &cargo,
    &helper_src_manifest,
    &helper_target_dir,
    "x86_64-apple-darwin",
    "tauri-cef-helper",
  );

  // Generate a small rust shim that exposes the embedded helper bytes.
  let shim_path = out_dir.join("cef_helpers.rs");
  let shim = format!(
    "pub const CEF_HELPER_AARCH64: &[u8] = include_bytes!(r#\"{}\"#);\n\
pub const CEF_HELPER_X86_64: &[u8] = include_bytes!(r#\"{}\"#);\n",
    aarch64.display(),
    x86_64.display()
  );
  fs::write(&shim_path, shim).expect("failed to write cef_helpers.rs");
}

/// Recursively copy every regular file under `src` into `dst`, mirroring
/// the directory layout. Used to materialise the cef-helper crate sources
/// into OUT_DIR so we can shell out to `cargo build` against a copy that
/// won't pollute the repo checkout with target/Cargo.lock artifacts.
fn copy_src_tree(src: &Path, dst: &Path) {
  for entry in fs::read_dir(src)
    .unwrap_or_else(|e| panic!("failed to read cef-helper src dir {}: {e}", src.display()))
  {
    let entry = entry.expect("failed to read cef-helper src entry");
    let from = entry.path();
    let to = dst.join(entry.file_name());
    let kind = entry
      .file_type()
      .expect("failed to stat cef-helper src entry");
    if kind.is_dir() {
      fs::create_dir_all(&to).expect("failed to mkdir for cef-helper src copy");
      copy_src_tree(&from, &to);
    } else if kind.is_file() {
      fs::copy(&from, &to).unwrap_or_else(|e| {
        panic!(
          "failed to copy cef-helper src {} -> {}: {e}",
          from.display(),
          to.display()
        )
      });
    }
  }
}

fn build_helper(
  cargo: &str,
  manifest_path: &Path,
  target_dir: &Path,
  target: &str,
  bin_name: &str,
) -> PathBuf {
  let mut cmd = Command::new(cargo);
  cmd
    .arg("build")
    .arg("--release")
    .arg("--manifest-path")
    .arg(manifest_path)
    .arg("--bin")
    .arg(bin_name)
    .arg("--target")
    .arg(target)
    .env("CARGO_TARGET_DIR", target_dir);

  let status = cmd
    .status()
    .expect("failed to spawn cargo build for CEF helper");
  if !status.success() {
    panic!("failed to build CEF helper for target {target}");
  }

  target_dir.join(target).join("release").join(bin_name)
}
