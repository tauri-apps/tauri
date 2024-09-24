// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  function __tauriDeepFreeze(object) {
    const props = Object.getOwnPropertyNames(object)

    for (const prop of props) {
      if (typeof object[prop] === 'object') {
        __tauriDeepFreeze(object[prop])
      }
    }

    return Object.freeze(object)
  }

  Object.defineProperty(window.__TAURI_INTERNALS__, '__TAURI_PATTERN__', {
    value: __tauriDeepFreeze(__TEMPLATE_pattern__)
  })
})()
