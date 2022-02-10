// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

;(function () {
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
        delete window[error]
      }, true)
      var error = window.__TAURI__.transformCallback(function (e) {
        reject(e)
        delete window[callback]
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

  // open <a href="..."> links with the Tauri API
  function __openLinks() {
    document.querySelector('body').addEventListener(
      'click',
      function (e) {
        var target = e.target
        while (target != null) {
          if (target.matches('a')) {
            if (
              target.href &&
              target.href.startsWith('http') &&
              target.target === '_blank'
            ) {
              window.__TAURI_INVOKE__('tauri', {
                __tauriModule: 'Shell',
                message: {
                  cmd: 'open',
                  path: target.href
                }
              })
              e.preventDefault()
            }
            break
          }
          target = target.parentElement
        }
      }
    )
  }

  if (
    document.readyState === 'complete' ||
    document.readyState === 'interactive'
  ) {
    __openLinks()
  } else {
    window.addEventListener(
      'DOMContentLoaded',
      function () {
        __openLinks()
      },
      true
    )
  }

  // drag region
  document.addEventListener('mousedown', (e) => {
    if (e.target.hasAttribute('data-tauri-drag-region') && e.buttons === 1) {
      // start dragging if the element has a `tauri-drag-region` data attribute and maximize on double-clicking it
      window.__TAURI_INVOKE__('tauri', {
        __tauriModule: 'Window',
        message: {
          cmd: 'manage',
          data: {
            cmd: {
              type: e.detail === 2 ? '__toggleMaximize' : 'startDragging'
            }
          }
        }
      })
    }
  })

  listen('tauri://window-created', function (event) {
    if (event.payload) {
      var windowLabel = event.payload.label
      window.__TAURI_METADATA__.__windows.push({
        label: windowLabel
      })
    }
  })

  let permissionSettable = false
  let permissionValue = 'default'

  function isPermissionGranted() {
    if (window.Notification.permission !== 'default') {
      return Promise.resolve(window.Notification.permission === 'granted')
    }
    return window.__TAURI_INVOKE__('tauri', {
      __tauriModule: 'Notification',
      message: {
        cmd: 'isNotificationPermissionGranted'
      }
    })
  }

  function setNotificationPermission(value) {
    permissionSettable = true
    window.Notification.permission = value
    permissionSettable = false
  }

  function requestPermission() {
    return window
      .__TAURI_INVOKE__('tauri', {
        __tauriModule: 'Notification',
        message: {
          cmd: 'requestNotificationPermission'
        }
      })
      .then(function (permission) {
        setNotificationPermission(permission)
        return permission
      })
  }

  function sendNotification(options) {
    if (typeof options === 'object') {
      Object.freeze(options)
    }

    return window.__TAURI_INVOKE__('tauri', {
      __tauriModule: 'Notification',
      message: {
        cmd: 'notification',
        options:
          typeof options === 'string'
            ? {
              title: options
            }
            : options
      }
    })
  }

  window.Notification = function (title, options) {
    var opts = options || {}
    sendNotification(
      Object.assign(opts, {
        title: title
      })
    )
  }

  window.Notification.requestPermission = requestPermission

  Object.defineProperty(window.Notification, 'permission', {
    enumerable: true,
    get: function () {
      return permissionValue
    },
    set: function (v) {
      if (!permissionSettable) {
        throw new Error('Readonly property')
      }
      permissionValue = v
    }
  })

  isPermissionGranted().then(function (response) {
    if (response === null) {
      setNotificationPermission('default')
    } else {
      setNotificationPermission(response ? 'granted' : 'denied')
    }
  })

  window.alert = function (message) {
    window.__TAURI_INVOKE__('tauri', {
      __tauriModule: 'Dialog',
      message: {
        cmd: 'messageDialog',
        message: message
      }
    })
  }

  window.confirm = function (message) {
    return window.__TAURI_INVOKE__('tauri', {
      __tauriModule: 'Dialog',
      message: {
        cmd: 'confirmDialog',
        message: message
      }
    })
  }

  // window.print works on Linux/Windows; need to use the API on macOS
  if (navigator.userAgent.includes('Mac')) {
    window.print = function () {
      return window.__TAURI_INVOKE__('tauri', {
        __tauriModule: 'Window',
        message: {
          cmd: 'manage',
          data: {
            cmd: {
              type: 'print'
            }
          }
        }
      })
    }
  }
})()
