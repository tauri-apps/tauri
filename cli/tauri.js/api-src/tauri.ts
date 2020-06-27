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

function invoke(args: any): void {
  window.external.invoke(typeof args === 'object' ? JSON.stringify(args) : args)
}

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
