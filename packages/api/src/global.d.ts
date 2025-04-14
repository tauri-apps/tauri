// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */

import type { invoke, transformCallback, convertFileSrc } from './core'

/** @ignore */
declare global {
  interface Window {
    __TAURI_INTERNALS__: {
      invoke: typeof invoke
      transformCallback: typeof transformCallback
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
