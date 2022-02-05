// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  function __tauriDeepFreeze(object) {
    const props = Object.getOwnPropertyNames(object)

    for (const prop of props) {
      if (typeof object[name] === 'object') {
        __tauriDeepFreeze(object[name])
      }
    }

    return Object.freeze(object)
  }

  Object.defineProperty(window, '__TAURI_PATTERN__', {
    value: __tauriDeepFreeze(__TEMPLATE_pattern__)
  })
})()
