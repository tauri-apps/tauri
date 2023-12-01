// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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

  /**
   * Invoke your custom commands.
   * @example
   * ```ts
   * import { invoke } from '@tauri-apps/api/tauri';
   * declare global {
   *  interface TauriInvokeTypes {
   *   myCustomCommand: (arg: { foo: string }) => Promise<{ bar: string }>
   * };
   * invoke('myCustomCommand', { foo: string }); // -> Promise<{ bar: string }>
   * ```
   */
  export interface TauriInvokeTypes {

  }
}

/** @ignore */
interface WindowDef {
  label: string
}
