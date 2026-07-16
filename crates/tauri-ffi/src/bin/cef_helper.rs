// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! CEF subprocess helper executable for the tauri-ffi cef distribution.
//!
//! CEF is multi-process: the browser process launches its renderer/GPU/utility
//! subprocesses by executing a helper program with a `--type=` switch. For a
//! Rust Tauri app that helper is the app binary itself — the `#[cef_entry_point]`
//! macro catches the re-exec and calls [`tauri::run_cef_helper_process`]. But a
//! bindings host (node/deno/python) can't act as a CEF subprocess, so the cef
//! `tauri-ffi` library points CEF's `browser_subprocess_path` at this dedicated
//! executable instead (see `TAURI_CEF_SUBPROCESS_PATH`).
//!
//! It is only ever launched by CEF as a subprocess — never run it directly. It
//! must be able to load `libcef` (staged next to it), so it is distributed
//! alongside `libtauri_cef`. Built only with `--features runtime-cef` (see the
//! `[[bin]]` `required-features` in Cargo.toml).

fn main() {
  tauri::run_cef_helper_process();
}
