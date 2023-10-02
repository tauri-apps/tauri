// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
  function uid() {
    return window.crypto.getRandomValues(new Uint32Array(1))[0]
  }

  const osName = __TEMPLATE_os_name__

  Object.defineProperties(window.__TAURI__.__INTERNALS__, 'convertFileSrc', {
    value: function (filePath, protocol = 'asset') {
      const path = encodeURIComponent(filePath)
      return osName === 'windows' || osName === 'android'
        ? `http://${protocol}.localhost/${path}`
        : `${protocol}://localhost/${path}`
    }
  })

  Object.defineProperties(window.__TAURI__.__INTERNALS__, 'transformCallback', {
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
    if (window.__TAURI__?.__INTERNALS__?.ipc) {
      for (const action of ipcQueue) {
        action()
      }
    } else {
      setTimeout(waitForIpc, 50)
    }
  }

  Object.defineProperties(window.__TAURI__.__INTERNALS__, 'invoke', {
    value: function (cmd, payload = {}, options) {
      return new Promise(function (resolve, reject) {
        const callback = window.__TAURI__.__INTERNALS__.transformCallback(
          function (r) {
            resolve(r)
            delete window[`_${error}`]
          },
          true
        )
        const error = window.__TAURI__.__INTERNALS__.transformCallback(
          function (e) {
            reject(e)
            delete window[`_${callback}`]
          },
          true
        )

        const action = () => {
          window.window.__TAURI__.__INTERNALS__.ipc({
            cmd,
            callback,
            error,
            payload,
            options
          })
        }
        if (window.__TAURI__?.__INTERNALS__?.ipc) {
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
