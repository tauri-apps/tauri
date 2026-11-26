// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */

import type { invoke, transformCallback, convertFileSrc } from './core'

/** @ignore */
declare global {
  interface Window {
    /**
     * The whole `@tauri-apps/api` package, exposed as a global object.
     *
     * Only defined when [`app.withGlobalTauri`](https://v2.tauri.app/reference/config/#withglobaltauri)
     * is set to `true` in `tauri.conf.json`. It is meant for vanilla JavaScript
     * frontends that do not use a bundler:
     *
     * ```js
     * const { event, window: tauriWindow, path } = window.__TAURI__;
     * ```
     */
    __TAURI__?: typeof import('./index')
    __TAURI_INTERNALS__: {
      invoke: typeof invoke
      transformCallback: typeof transformCallback
      unregisterCallback: (id: number) => void
      runCallback: (id: number, data: unknown) => void
      callbacks: Map<number, (data: unknown) => void>
      convertFileSrc: typeof convertFileSrc
      ipc: (message: {
        cmd: string
        callback: number
        error: number
        payload: unknown
        options?: InvokeOptions
      }) => void
      metadata: {
        currentWindow: WindowDef
        currentWebview: WebviewDef
      }
      plugins: {
        path: {
          sep: string
          delimiter: string
        }
      }
    }
    __TAURI_EVENT_PLUGIN_INTERNALS__: {
      unregisterListener: (event: string, eventId: number) => void
    }
  }
}

/** @ignore */
interface WebviewDef {
  windowLabel: string
  label: string
}

/** @ignore */
interface WindowDef {
  label: string
}
