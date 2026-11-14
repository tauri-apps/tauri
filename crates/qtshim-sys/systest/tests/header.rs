// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Header <-> Rust checks that ctest doesn't do: function and callback signatures, struct field
//! types, which declarations exist on each side, and that enum constants are distinct.

use std::collections::{BTreeMap, BTreeSet};

use qtshim_sys as sys;
use qtshim_sys_systest::*;

/// Functions the header declares only for the `webengine` feature (its `[webengine]` sections).
fn is_webengine_fn(name: &str) -> bool {
  ["qs_window_", "qs_view_", "qs_profile_", "qs_job_"]
    .iter()
    .any(|p| name.starts_with(p))
    || matches!(
      name,
      "qs_set_scheme_handler"
        | "qs_register_scheme"
        | "qs_engine_add_flags"
        | "qs_engine_started"
        | "qs_engine_version"
        | "qs_set_proxy"
    )
}

/// Header enum groups whose values only the safe `qtshim` crate (or C++) uses: `qtshim-sys` has
/// no constants for them.
const UNBOUND_PREFIXES: &[&str] = &["QS_WIN_", "QS_STATE_", "QS_FLAG_", "QS_VIEW_"];

fn set<'a>(items: &[&'a str]) -> BTreeSet<&'a str> {
  items.iter().copied().collect()
}

#[test]
fn signatures_and_field_types_match_header() {
  assert!(
    SIGCHECK_LOG.trim().is_empty(),
    "Rust declarations differ from qtshim.h (generated sigcheck.cpp):\n{SIGCHECK_LOG}"
  );
}

#[test]
fn every_rust_fn_is_declared_in_header() {
  let extra: Vec<_> = set(RUST_FNS)
    .difference(&set(HEADER_FNS))
    .copied()
    .collect();
  assert!(
    extra.is_empty(),
    "declared in qtshim-sys but not in qtshim.h: {extra:?}"
  );
}

#[test]
fn every_header_fn_is_bound() {
  let missing: Vec<_> = set(HEADER_FNS)
    .difference(&set(RUST_FNS))
    .copied()
    .filter(|f| cfg!(feature = "webengine") || !is_webengine_fn(f))
    .collect();
  assert!(
    missing.is_empty(),
    "declared in qtshim.h but missing from qtshim-sys: {missing:?}"
  );
}

#[test]
#[cfg(not(feature = "webengine"))]
fn webengine_fns_are_gated() {
  let leaked: Vec<_> = RUST_FNS.iter().filter(|f| is_webengine_fn(f)).collect();
  assert!(
    leaked.is_empty(),
    "webengine functions declared without the feature: {leaked:?}"
  );
}

#[test]
fn callbacks_match_header() {
  let header: BTreeSet<_> = HEADER_CALLBACKS
    .iter()
    .copied()
    .filter(|c| cfg!(feature = "webengine") || *c != "qs_scheme_fn")
    .collect();
  assert_eq!(set(RUST_CALLBACKS), header);
}

#[test]
fn rust_consts_match_header_values() {
  let header: BTreeMap<_, _> = header_consts().into_iter().collect();
  assert!(!RUST_CONSTS.is_empty());
  for (name, value) in RUST_CONSTS {
    let c = header
      .get(name)
      .unwrap_or_else(|| panic!("{name} is not in qtshim.h"));
    assert_eq!(value, c, "{name}");
  }
}

#[test]
fn every_header_const_is_bound_or_known_unbound() {
  let rust: BTreeSet<_> = RUST_CONSTS.iter().map(|(n, _)| *n).collect();
  let missing: Vec<_> = header_consts()
    .into_iter()
    .map(|(n, _)| n)
    .filter(|n| !rust.contains(n) && !UNBOUND_PREFIXES.iter().any(|p| n.starts_with(p)))
    .collect();
  assert!(
    missing.is_empty(),
    "qtshim.h constants missing from qtshim-sys: {missing:?}"
  );
}

#[test]
fn header_enum_values_are_distinct() {
  for group in HEADER_ENUMS {
    let mut seen = BTreeMap::new();
    for name in *group {
      let v = header_const(name);
      if let Some(prev) = seen.insert(v, name) {
        panic!("{prev} and {name} are both {v}");
      }
    }
  }
}

#[test]
fn header_bit_flags_are_single_disjoint_bits() {
  for prefix in ["QS_STATE_", "QS_VIEW_", "QS_WIN_"] {
    let mut all = 0i64;
    let names: Vec<_> = header_consts()
      .into_iter()
      .filter(|(n, _)| n.starts_with(prefix))
      .collect();
    assert!(!names.is_empty(), "no {prefix}* constants");
    for (name, v) in names {
      assert!(
        v > 0 && v & (v - 1) == 0,
        "{name} = {v} is not a single bit"
      );
      assert_eq!(all & v, 0, "{name} overlaps another {prefix}* bit");
      all |= v;
    }
  }
}

#[test]
fn event_kinds_are_distinct() {
  let kinds = [
    sys::QS_EV_WINDOW_RESIZED,
    sys::QS_EV_WINDOW_MOVED,
    sys::QS_EV_WINDOW_CLOSE_REQ,
    sys::QS_EV_WINDOW_FOCUS,
    sys::QS_EV_WINDOW_SCALE,
    sys::QS_EV_THEME,
    sys::QS_EV_VIEW_IPC,
    sys::QS_EV_VIEW_TITLE,
    sys::QS_EV_VIEW_LOAD,
    sys::QS_EV_VIEW_NAVIGATE,
    sys::QS_EV_VIEW_NEW_WINDOW,
    sys::QS_EV_VIEW_PERMISSION,
    sys::QS_EV_VIEW_DOWNLOAD,
    sys::QS_EV_VIEW_DOWNLOAD_DONE,
    sys::QS_EV_VIEW_DRAG,
    sys::QS_EV_VIEW_CRASHED,
    sys::QS_EV_ABOUT_TO_BLOCK,
  ];
  let unique: BTreeSet<_> = kinds.iter().collect();
  assert_eq!(unique.len(), kinds.len());
  // 0 would be indistinguishable from a zeroed event.
  assert!(kinds.iter().all(|k| *k > 0));
  // The list above is complete.
  let rust_ev = RUST_CONSTS
    .iter()
    .filter(|(n, _)| n.starts_with("QS_EV_"))
    .count();
  assert_eq!(rust_ev, kinds.len());
}

#[test]
fn standard_actions_are_distinct() {
  let kinds = [
    sys::QS_ACTION_COPY,
    sys::QS_ACTION_CUT,
    sys::QS_ACTION_PASTE,
    sys::QS_ACTION_SELECT_ALL,
    sys::QS_ACTION_UNDO,
    sys::QS_ACTION_REDO,
    sys::QS_ACTION_DELETE,
    sys::QS_ACTION_MINIMIZE,
    sys::QS_ACTION_MAXIMIZE,
    sys::QS_ACTION_FULLSCREEN,
    sys::QS_ACTION_HIDE,
    sys::QS_ACTION_CLOSE_WINDOW,
    sys::QS_ACTION_QUIT,
  ];
  let unique: BTreeSet<_> = kinds.iter().collect();
  assert_eq!(unique.len(), kinds.len());
  assert!(kinds.iter().all(|k| *k > 0));
  let rust_actions = RUST_CONSTS
    .iter()
    .filter(|(n, _)| n.starts_with("QS_ACTION_"))
    .count();
  assert_eq!(rust_actions, kinds.len());
}

/// `qtshim` hard-codes these header values instead of going through `qtshim-sys`.
#[test]
#[cfg(feature = "webengine")]
fn qtshim_window_state_bits_match_header() {
  use qtshim::webengine::window_state as s;
  let pairs = [
    (s::MINIMIZED, "QS_STATE_MINIMIZED"),
    (s::MAXIMIZED, "QS_STATE_MAXIMIZED"),
    (s::FULLSCREEN, "QS_STATE_FULLSCREEN"),
    (s::ACTIVE, "QS_STATE_ACTIVE"),
    (s::VISIBLE, "QS_STATE_VISIBLE"),
    (s::ENABLED, "QS_STATE_ENABLED"),
    (s::DECORATED, "QS_STATE_DECORATED"),
    (s::ON_TOP, "QS_STATE_ON_TOP"),
    (s::RESIZABLE, "QS_STATE_RESIZABLE"),
    (s::CLOSABLE, "QS_STATE_CLOSABLE"),
    (s::MINIMIZABLE, "QS_STATE_MINIMIZABLE"),
    (s::MAXIMIZABLE, "QS_STATE_MAXIMIZABLE"),
  ];
  for (rust, name) in pairs {
    assert_eq!(rust as i64, header_const(name), "{name}");
  }
  let header = header_consts()
    .iter()
    .filter(|(n, _)| n.starts_with("QS_STATE_"))
    .count();
  assert_eq!(
    header,
    pairs.len(),
    "window_state is missing a QS_STATE_* bit"
  );
}

/// `qtshim` hard-codes these header values instead of going through `qtshim-sys`.
#[test]
#[cfg(feature = "webengine")]
fn qtshim_window_flags_match_header() {
  use qtshim::webengine::WindowFlag as F;
  let pairs = [
    (F::Decorations, "QS_FLAG_DECORATIONS"),
    (F::AlwaysOnTop, "QS_FLAG_ON_TOP"),
    (F::AlwaysOnBottom, "QS_FLAG_ON_BOTTOM"),
    (F::Focusable, "QS_FLAG_FOCUSABLE"),
    (F::Closable, "QS_FLAG_CLOSABLE"),
    (F::Minimizable, "QS_FLAG_MINIMIZABLE"),
    (F::Maximizable, "QS_FLAG_MAXIMIZABLE"),
    (F::IgnoreCursorEvents, "QS_FLAG_IGNORE_CURSOR"),
    (F::SkipTaskbar, "QS_FLAG_SKIP_TASKBAR"),
  ];
  for (rust, name) in pairs {
    assert_eq!(rust as i64, header_const(name), "{name}");
  }
  let header = header_consts()
    .iter()
    .filter(|(n, _)| n.starts_with("QS_FLAG_"))
    .count();
  assert_eq!(
    header,
    pairs.len(),
    "WindowFlag is missing a QS_FLAG_* value"
  );
}
