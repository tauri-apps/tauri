declare global {
  interface Window {
    __TAURI_INVOKE_HANDLER__: (command: string) => void
  }
}

function s4(): string {
  return Math.floor((1 + Math.random()) * 0x10000)
    .toString(16)
    .substring(1)
}

function uid(): string {
  return (
    s4() +
    s4() +
    '-' +
    s4() +
    '-' +
    s4() +
    '-' +
    s4() +
    '-' +
    s4() +
    s4() +
    s4()
  )
}

/**
 * sends a synchronous command to the backend
 *
 * @param args
 */
function invoke(args: any): void {
  window.__TAURI_INVOKE_HANDLER__(args)
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
    writable: false
  })

  return identifier
}

/**
 * sends an asynchronous command to the backend
 *
 * @param args
 *
 * @return {Promise<T>} Promise resolving or rejecting to the backend response
 */
async function promisified<T>(args: any): Promise<T> {
  return await new Promise((resolve, reject) => {
    const callback = transformCallback((e) => {
      resolve(e)
      Reflect.deleteProperty(window, error)
    }, true)
    const error = transformCallback((e) => {
      reject(e)
      Reflect.deleteProperty(window, callback)
    }, true)

    invoke({
      callback,
      error,
      ...args
    })
  })
}

export { invoke, transformCallback, promisified }
