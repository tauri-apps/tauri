// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The examples/api app is built with `withGlobalTauri: true`, so the whole
// `@tauri-apps/api` surface is available on `window.__TAURI__` inside the webview.
// This mirrors that for the functions we serialize and run in the page.

import type * as TauriApi from '@tauri-apps/api'

declare global {
  interface Window {
    __TAURI__: typeof TauriApi
  }
}

export {}
