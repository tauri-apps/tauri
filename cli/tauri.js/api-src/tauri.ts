declare global {
  interface External {
    invoke: (command: string) => void
  }
}

function s4(): string {
  return Math.floor((1 + Math.random()) * 0x10000)
    .toString(16)
    .substring(1)
}

function uid(): string {
  return s4() + s4() + '-' + s4() + '-' + s4() + '-' +
    s4() + '-' + s4() + s4() + s4()
}

/**
 * Invokes a command with the given argument serialized to string.
 *
 * @param {any} srgs the command definition object/string.
 */
function invoke(args: any): void {
  window.external.invoke(typeof args === 'object' ? JSON.stringify(args) : args)
}

/**
 * Transforms a callback to be used as a String by Rust.
 *
 * @param {Function} callback the callback to transform.
 *
 * @return {string} the String representing the callback function name.
 */
function transformCallback(callback?: (response: any) => void, once = false): string {
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
 * Invokes an async command with the given argument serialized to string.
 * Meant to be used with `tauri::execute_promise`.
 *
 * @param {any} args the command definition object/string.
 *
 * @return {Promise<T>} promise resolving/rejecting to the Rust response.
 */
async function promisified<T>(args: any): Promise<T> {
  return await new Promise((resolve, reject) => {
    invoke({
      callback: transformCallback(resolve),
      error: transformCallback(reject),
      ...args
    })
  })
}

export {
  invoke,
  transformCallback,
  promisified
}
