// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  function uid() {
    return window.crypto.getRandomValues(new Uint32Array(1))[0]
  }

  const osName = __TEMPLATE_os_name__
  const protocolScheme = __TEMPLATE_protocol_scheme__

  Object.defineProperty(window.__TAURI_INTERNALS__, 'convertFileSrc', {
    value: function (filePath, protocol = 'asset') {
      const path = encodeURIComponent(filePath)
      return osName === 'windows' || osName === 'android'
        ? `${protocolScheme}://${protocol}.localhost/${path}`
        : `${protocol}://localhost/${path}`
    }
  })

  Object.defineProperty(window.__TAURI_INTERNALS__, 'transformCallback', {
    value: function transformCallback(callback, once) {
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
  })

  const ipcQueue = []
  let isWaitingForIpc = false

  function waitForIpc() {
    if ('ipc' in window.__TAURI_INTERNALS__) {
      for (const action of ipcQueue) {
        action()
      }
    } else {
      setTimeout(waitForIpc, 50)
    }
  }

  Object.defineProperty(window.__TAURI_INTERNALS__, 'invoke', {
    value: function (cmd, payload = {}, options) {
      return new Promise(function (resolve, reject) {
        const callback = window.__TAURI_INTERNALS__.transformCallback(function (
          r
        ) {
          resolve(r)
          delete window[`_${error}`]
        }, true)
        const error = window.__TAURI_INTERNALS__.transformCallback(function (
          e
        ) {
          reject(e)
          delete window[`_${callback}`]
        }, true)

        const action = () => {
          window.__TAURI_INTERNALS__.ipc({
            cmd,
            callback,
            error,
            payload,
            options
          })
        }
        if ('ipc' in window.__TAURI_INTERNALS__) {
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
  })
})()
