// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

declare global {
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  interface Window {
    rpc: {
      notify: (command: string, args?: { [key: string]: unknown }) => void
    }
  }
}

function uid(): string {
  const length = new Int8Array(1)
  window.crypto.getRandomValues(length)
  const array = new Uint8Array(Math.max(16, Math.abs(length[0])))
  window.crypto.getRandomValues(array)
  return array.join('')
}

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

export interface InvokeArgs {
  mainThread?: boolean
  [key: string]: unknown
}

/**
 * sends a message to the backend
 *
 * @param args
 *
 * @return {Promise<T>} Promise resolving or rejecting to the backend response
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
