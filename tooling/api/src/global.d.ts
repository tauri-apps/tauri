// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import type { invoke, transformCallback, convertFileSrc } from './primitives'

/** @ignore */
declare global {
  interface Window {
    __TAURI__: {
      __INTERNALS__: {
        invoke: typeof invoke
        transformCallback: typeof transformCallback
        convertFileSrc: typeof convertFileSrc
        ipc: (message: any) => void
        metadata: {
          windows: WindowDef[]
          currentWindow: WindowDef
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
}
