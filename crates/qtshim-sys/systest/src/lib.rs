// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Checks that `qtshim-sys` (hand-written FFI declarations) matches `cpp/qtshim.h`.
//!
//! Run with `cargo test -p qtshim-sys-systest` (core API) and
//! `cargo test -p qtshim-sys-systest --features webengine` (everything). See `build.rs` for what
//! is generated; the checks themselves live in `tests/`.

// Link qtshim-sys (the C++ shim) into every test of this crate.
pub use qtshim_sys;

include!(concat!(env!("OUT_DIR"), "/tables.rs"));

/// Compiler diagnostics from the generated signature / field type checks (`sigcheck.cpp`);
/// empty when Rust and the header agree.
pub const SIGCHECK_LOG: &str = include_str!(concat!(env!("OUT_DIR"), "/sigcheck.log"));

unsafe extern "C" {
  static qs_systest_header_consts: [i64; HEADER_CONST_COUNT + 1];
}

/// Every `QS_*` enumerator of the header with its value (as computed by the C compiler).
pub fn header_consts() -> Vec<(&'static str, i64)> {
  let values = unsafe { &qs_systest_header_consts };
  HEADER_ENUMS
    .iter()
    .flat_map(|group| group.iter().copied())
    .zip(values.iter().copied())
    .collect()
}

/// The value of the header enumerator `name`.
pub fn header_const(name: &str) -> i64 {
  header_consts()
    .into_iter()
    .find(|(n, _)| *n == name)
    .unwrap_or_else(|| panic!("{name} is not declared in qtshim.h"))
    .1
}
