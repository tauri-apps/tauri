// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  if (!window.__TAURI__) {
    Object.defineProperty(window, '__TAURI__', {
      value: {}
    })
  }

  if (!window.__TAURI__.__INTERNALS__) {
    Object.defineProperty(window.__TAURI__, '__INTERNALS__', {
      value: {
        plugins: {}
      }
    })
  }

  __RAW_freeze_prototype__

  __RAW_pattern_script__

  __RAW_ipc_script__

  __RAW_listen_function__

  __RAW_core_script__

  __RAW_event_initialization_script__
  ;(function () {
    __RAW_bundle_script__
  })()

  if (window.ipc) {
    window.__TAURI__.__INTERNALS__.invoke('__initialized', {
      url: window.location.href
    })
  } else {
    window.addEventListener('DOMContentLoaded', function () {
      window.__TAURI__.__INTERNALS__.invoke('__initialized', {
        url: window.location.href
      })
    })
  }

  __RAW_plugin_initialization_script__
})()
