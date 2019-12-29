const cache = {}
let initialized = false

const proxy = new Proxy({
  __consume () {
    for (const key in cache) {
      if (key in window.tauri) {
        const queue = cache[key]
        for (const call of queue) {
          try {
            const ret = window.tauri[key].apply(window.tauri, call.arguments)
            if (ret instanceof Promise) {
              ret.then(call.resolve, call.reject)
            } else {
              call.resolve()
            }
          } catch (e) {
            call.reject(e)
          }
        }
      }
    }
    initialized = true
  }
}, {
  get (obj, prop) {
    if (prop === '__consume') {
      return obj[prop]
    }

    if (initialized && 'tauri' in window) {
      return window.tauri[prop].bind(window.tauri)
    }

    if (!(prop in cache)) {
      cache[prop] = []
    }
    return function () {
      const promise = new Promise((resolve, reject) => {
        cache[prop].push({
          arguments: arguments,
          resolve,
          reject
        })
      });
      return promise;
    }

  }
})

window.onTauriInit = () => {
  proxy.__consume()
}

export default proxy
