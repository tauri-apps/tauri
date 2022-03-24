// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */

function isLinux(): boolean {
  return navigator.appVersion.includes('Linux')
}

function isWindows(): boolean {
  return navigator.appVersion.includes('Win')
}

function isMacOS(): boolean {
  return navigator.appVersion.includes('Mac')
}

export { isLinux, isWindows, isMacOS }
