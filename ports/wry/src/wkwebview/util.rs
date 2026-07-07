// Copyright 2020-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use objc2_foundation::NSProcessInfo;

pub fn operating_system_version() -> (isize, isize, isize) {
  let process_info = NSProcessInfo::processInfo();
  let version = process_info.operatingSystemVersion();
  (
    version.majorVersion,
    version.minorVersion,
    version.patchVersion,
  )
}
