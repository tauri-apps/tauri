declare global {
  interface External {
    invoke(command: string): void
  }
}

function __s4(): string {
  return Math.floor((1 + Math.random()) * 0x10000)
    .toString(16)
    .substring(1)
}

function __uid(): string {
  return __s4() + __s4() + '-' + __s4() + '-' + __s4() + '-' +
    __s4() + '-' + __s4() + __s4() + __s4()
}

function invoke(args: any) {
  window.external.invoke(typeof args === 'object' ? JSON.stringify(args) : args)
}

function transformCallback(callback: (response: any) => void, once = false) {
  var identifier = __uid();

  Object.defineProperty(window, identifier, {
    value: (result: any) => {
      if (once) {
        Reflect.deleteProperty(window, identifier)
      }

      return callback && callback(result)
    },
    writable: false
  })

  return identifier
}

function promisified(args: any): Promise<any> {
  return new Promise((resolve, reject) => {
    invoke({
      callback: transformCallback(resolve),
      error: transformCallback(reject),
      ...args
    })
  });
}

// init tauri API
try {
  invoke({
    cmd: 'init'
  })
} catch (e) {
  window.addEventListener('DOMContentLoaded', function () {
    invoke({
      cmd: 'init'
    })
  }, true)
}

export {
  invoke,
  transformCallback,
  promisified
}
