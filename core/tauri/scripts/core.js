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

  const osName = __TEMPLATE_os_name__

  window.__TAURI__.convertFileSrc = function convertFileSrc(filePath, protocol = 'asset') {
    const path = encodeURIComponent(filePath)
    return osName === 'windows' || osName === 'android'
      ? `http://${protocol}.localhost/${path}`
      : `${protocol}://localhost/${path}`
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

  window.__TAURI_INVOKE__ = function invoke(cmd, payload = {}, options) {
    return new Promise(function (resolve, reject) {
      const callback = window.__TAURI__.transformCallback(function (r) {
        resolve(r)
        delete window[`_${error}`]
      }, true)
      const error = window.__TAURI__.transformCallback(function (e) {
        reject(e)
        delete window[`_${callback}`]
      }, true)

      const action = () => {
        window.__TAURI_IPC__({
          cmd,
          callback,
          error,
          payload,
          options
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
