// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! `qtshim::webengine`, one file per area.

use qtshim::test_harness::Case;

mod profile;
mod scheme;
mod view;
mod window;

pub fn cases() -> Vec<Case> {
  let mut cases = Vec::new();
  cases.extend(window::cases());
  cases.extend(view::cases());
  cases.extend(scheme::cases());
  cases.extend(profile::cases());
  cases
}
