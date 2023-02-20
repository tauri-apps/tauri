// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */

import { type InvokeArgs, transformCallback } from '../tauri'

type TauriModule =
  | 'App'
  | 'Fs'
  | 'Path'
  | 'Os'
  | 'Window'
  | 'Shell'
  | 'Event'
  | 'Internal'
  | 'Dialog'
  | 'Cli'
  | 'Notification'
  | 'Http'
  | 'GlobalShortcut'
  | 'Process'
  | 'Clipboard'

async function invokeTauriCommand<T>(
  module: TauriModule,
  command: Record<any, any>
): Promise<T> {
  return invokeInternal('tauri', command, module)
}

async function invokeInternal<T>(
  cmd: string,
  args: InvokeArgs = {},
  tauriModule?: TauriModule
): Promise<T> {
  return new Promise((resolve, reject) => {
    const callback = transformCallback((e: T) => {
      resolve(e)
      Reflect.deleteProperty(window, `_${error}`)
    }, true)
    const error = transformCallback((e) => {
      reject(e)
      Reflect.deleteProperty(window, `_${callback}`)
    }, true)

    window.__TAURI_IPC__({
      cmd,
      callback,
      error,
      args,
      tauri_module: tauriModule
    })
  })
}

export type { TauriModule }

export { invokeTauriCommand, invokeInternal }
