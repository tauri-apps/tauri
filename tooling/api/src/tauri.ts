// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Invoke your custom commands.
 *
 * This package is also accessible with `window.__TAURI__.tauri` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 * @packageDocumentation
 */

/** @ignore */
declare global {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface Window {
    rpc: {
      notify: (command: string, args?: { [key: string]: unknown }) => void
    }
  }
}

/** @ignore */
function uid(): string {
  const length = new Int8Array(1)
  window.crypto.getRandomValues(length)
  const array = new Uint8Array(Math.max(16, Math.abs(length[0])))
  window.crypto.getRandomValues(array)
  return array.join('')
}

/**
 * Transforms a callback function to a string identifier that can be passed to the backend.
 * The backend uses the identifier to `eval()` the callback.
 *
 * @return A unique identifier associated with the callback function.
 */
function transformCallback(
  callback?: (response: any) => void,
  once = false
): string {
  const identifier = uid()

  Object.defineProperty(window, identifier, {
    value: (result: any) => {
      if (once) {
        Reflect.deleteProperty(window, identifier)
      }

      return callback?.(result)
    },
    writable: false,
    configurable: true
  })

  return identifier
}

/** Command arguments. */
interface InvokeArgs {
  [key: string]: unknown
}

/**
 * Sends a message to the backend.
 *
 * @param cmd The command name.
 * @param args The optional arguments to pass to the command.
 * @return A promise resolving or rejecting to the backend response.
 */
async function invoke<T>(cmd: string, args: InvokeArgs = {}): Promise<T> {
  return new Promise((resolve, reject) => {
    const callback = transformCallback((e) => {
      resolve(e)
      Reflect.deleteProperty(window, error)
    }, true)
    const error = transformCallback((e) => {
      reject(e)
      Reflect.deleteProperty(window, callback)
    }, true)

    window.rpc.notify(cmd, {
      // @ts-expect-error the `__TAURI_INVOKE_KEY__` variable is injected at runtime by Tauri
      __invokeKey: __TAURI_INVOKE_KEY__,
      callback,
      error,
      ...args
    })
  })
}

/**
 * Convert a device file path to an URL that can be loaded by the webview.
 * Note that `asset:` must be allowed on the `csp` value configured on `tauri.conf.json`.
 *
 * @param  filePath the file path. On Windows, the drive name must be omitted, i.e. using `/Users/user/file.png` instead of `C:/Users/user/file.png`.
 *
 * @return the URL that can be used as source on the webview
 */
function convertFileSrc(filePath: string): string {
  return navigator.userAgent.includes('Windows')
    ? `https://custom.protocol.asset_${filePath}`
    : `asset://${filePath}`
}

export type { InvokeArgs }

export { transformCallback, invoke, convertFileSrc }
