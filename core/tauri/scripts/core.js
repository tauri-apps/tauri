// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

; (function () {
  function uid() {
    return window.crypto.getRandomValues(new Uint32Array(1))[0]
  }

  if (!window.__TAURI__) {
    Object.defineProperty(window, '__TAURI__', {
      value: {}
    })
  }

  window.__TAURI__.transformCallback = function transformCallback(
    callback,
    once
  ) {
    var identifier = uid()
    var prop = `_${identifier}`

    Object.defineProperty(window, prop, {
      value: (result) => {
        if (once) {
          Reflect.deleteProperty(window, prop)
        }

        return callback && callback(result)
      },
      writable: false,
      configurable: true
    })

    return identifier
  }

  const ipcQueue = []
  let isWaitingForIpc = false

  function waitForIpc() {
    if ('__TAURI_IPC__' in window) {
      for (const action of ipcQueue) {
        action()
      }
    } else {
      setTimeout(waitForIpc, 50)
    }
  }

  window.__TAURI_INVOKE__ = function invoke(cmd, args = {}) {
    return new Promise(function (resolve, reject) {
      var callback = window.__TAURI__.transformCallback(function (r) {
        resolve(r)
        delete window[`_${error}`]
      }, true)
      var error = window.__TAURI__.transformCallback(function (e) {
        reject(e)
        delete window[`_${callback}`]
      }, true)

      if (typeof cmd === 'string') {
        args.cmd = cmd
      } else if (typeof cmd === 'object') {
        args = cmd
      } else {
        return reject(new Error('Invalid argument type.'))
      }

      const action = () => {
        window.__TAURI_IPC__({
          ...args,
          callback,
          error: error
        })
      }
      if (window.__TAURI_IPC__) {
        action()
      } else {
        ipcQueue.push(action)
        if (!isWaitingForIpc) {
          waitForIpc()
          isWaitingForIpc = true
        }
      }
    })
  }
})()
