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

  const callbacks = new Map()

  function registerCallback(callback, once) {
    const identifier = uid()
    callbacks.set(identifier, (data) => {
      if (once) {
        unregisterCallback(identifier)
      }
      return callback && callback(data)
    })
    return identifier
  }

  function unregisterCallback(id) {
    callbacks.delete(id)
  }

  // Maybe let's rename it to `transformCallback`?
  Object.defineProperty(window.__TAURI_INTERNALS__, 'transformCallback', {
    value: registerCallback
  })

  Object.defineProperty(window.__TAURI_INTERNALS__, 'unregisterCallback', {
    value: unregisterCallback
  })

  Object.defineProperty(window.__TAURI_INTERNALS__, 'runCallback', {
    value: function runCallback(id, data) {
      const callback = callbacks.get(id)
      if (callback) {
        callback(data)
      } else {
        console.warn(
          `[TAURI] Couldn't find callback id ${id} in window. This might happen when the app is reloaded while Rust is running an asynchronous operation.`
        )
      }
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
        const callback = window.__TAURI_INTERNALS__.transformCallback((r) => {
          resolve(r)
          unregisterCallback(error)
        }, true)
        const error = window.__TAURI_INTERNALS__.transformCallback((e) => {
          reject(e)
          unregisterCallback(callback)
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
