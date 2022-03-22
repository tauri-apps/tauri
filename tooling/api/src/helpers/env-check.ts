// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */

function isBrowser(): boolean {
  return typeof window !== 'undefined'
}

function isLinux(): boolean {
  if (isBrowser()) {
    return navigator.appVersion.includes('Linux')
  } else {
    return process.platform === 'linux'
  }
}

function isWindows(): boolean {
  if (isBrowser()) {
    return navigator.appVersion.includes('Win')
  } else {
    return process.platform === 'win32'
  }
}

function isMacOS(): boolean {
  if (isBrowser()) {
    return navigator.appVersion.includes('Mac')
  } else {
    return process.platform === 'darwin'
  }
}

export { isLinux, isWindows, isMacOS, isBrowser }
