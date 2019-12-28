const proxy = new Proxy({
  __consume () {
    this.__consuming = true
    for (const key in this) {
      if (key in window.tauri) {
        const cache = this[key]
        for (const call of cache) {
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
    this.__consuming = false
  }
}, {
  set (obj, prop, value) {
    obj[prop] = value
    return true
  },
  get (obj, prop) {
    if (obj.__consuming || prop === '__consume') {
      return obj[prop]
    }

    if ('tauri' in window) {
      return window.tauri[prop].bind(window.tauri)
    }
    if (!(prop in obj)) {
      obj[prop] = []
    }
    return function () {
      const promise = new Promise((resolve, reject) => {
        obj[prop].push({
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
