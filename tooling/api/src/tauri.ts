// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Invoke your custom commands.
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
export interface InvokeArgs {
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
      callback,
      error,
      ...args
    })
  })
}

export { transformCallback, invoke }
