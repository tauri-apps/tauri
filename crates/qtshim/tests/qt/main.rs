// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Tests that run a real Qt application (offscreen), one process per case.
//!
//! `cargo test -p qtshim --features webengine,test-harness --test qt [filter]`
//!
//! Needs the `webengine` feature: it's the only way qtshim creates widgets (windows).
//!
//! Each module returns its cases from `cases()`; add new modules to `main` below.

// The cases clone handles on purpose (to test `Clone`, or to keep one alive in a closure).
#![allow(clippy::redundant_clone, clippy::assigning_clones)]

mod common;

mod app;
mod lifecycle;
mod menu;
mod webengine;

fn main() {
  let mut cases = Vec::new();
  cases.extend(app::cases());
  cases.extend(lifecycle::cases());
  cases.extend(menu::cases());
  cases.extend(webengine::cases());
  qtshim::test_harness::main(cases, common::teardown);
}
