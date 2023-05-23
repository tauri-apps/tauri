// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to operating system operations.

/// Returns `Some(String)` with a `BCP-47` language tag inside. If the locale couldn’t be obtained, `None` is returned instead.
pub fn locale() -> Option<String> {
  sys_locale::get_locale()
}
