// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tests that drive `tauri-runtime-qt` through the `tauri_runtime` traits with a real Qt
//! application (offscreen), one process per case.
//!
//! `cargo test -p tauri-runtime-qt --test qt [filter]`
//!
//! Each module returns its cases from `cases()`; add new modules to `main` below.

// The cases clone handles on purpose (to test `Clone`, or to keep one alive in a closure).
#![allow(clippy::redundant_clone, clippy::assigning_clones)]

mod common;

mod exit;
mod runtime;
mod webview;
mod window;

fn main() {
  let mut cases = Vec::new();
  cases.extend(runtime::cases());
  cases.extend(exit::cases());
  cases.extend(window::cases());
  cases.extend(webview::cases());
  qtshim::test_harness::main(cases, common::teardown);
}
