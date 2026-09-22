// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// The example is built with `withGlobalTauri: true`, so the `@tauri-apps/api`
// surface is on `window.__TAURI__` inside every page. Only the part the specs
// evaluate in the page is typed here.

declare global {
  interface Window {
    __TAURI__: {
      webview: {
        getAllWebviews(): Promise<{ label: string }[]>
      }
    }
  }
}

export {}
