// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function() {
  __RAW_freeze_prototype__

  ;(function() {
    __RAW_hotkeys__
  })()

  __RAW_pattern_script__

  __RAW_ipc_script__
  ;(function() {
    __RAW_bundle_script__
  })()

  __RAW_listen_function__

  __RAW_core_script__

  __RAW_event_initialization_script__

  if(window.__TAURI_INVOKE__ !== undefined) window.__TAURI_INVOKE__('__initialized', { url: window.location.href })
})()
