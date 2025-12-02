// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Native iOS path resolution using Foundation APIs via objc2-foundation.

use objc2_foundation::{
  NSSearchPathDirectory, NSSearchPathDomainMask, NSSearchPathForDirectoriesInDomains,
};
use std::path::PathBuf;

// Apple Foundation constants (NSSearchPathDirectory)
const NS_LIBRARY_DIRECTORY: NSSearchPathDirectory = NSSearchPathDirectory(5);

// NSUserDomainMask = 1
const NS_USER_DOMAIN_MASK: NSSearchPathDomainMask = NSSearchPathDomainMask::from_bits_retain(1);

/// Returns the app's library directory (equivalent to FileManager.urls(for: .libraryDirectory, ...)).
pub fn app_library_dir() -> Option<PathBuf> {
  resolve_search_path_directory(NS_LIBRARY_DIRECTORY)
}

fn resolve_search_path_directory(directory: NSSearchPathDirectory) -> Option<PathBuf> {
  let arr = unsafe { NSSearchPathForDirectoriesInDomains(directory, NS_USER_DOMAIN_MASK, true) };
  let first = unsafe { arr.firstObject_unchecked() }?;
  let path = first.to_string();
  if path.is_empty() {
    return None;
  }
  Some(PathBuf::from(path))
}
