// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// eslint-disable-next-line
Object.defineProperty(window, '__TAURI_EVENT_PLUGIN_INTERNALS__', {
  value: {
    unregisterListener: __RAW_unregister_listener_function__
  }
})
