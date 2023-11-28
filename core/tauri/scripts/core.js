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
  const protocolScheme = __TEMPLATE_protocol_scheme__

  window.__TAURI__.convertFileSrc = function convertFileSrc(filePath, protocol = 'asset') {
    const path = encodeURIComponent(filePath)
    return osName === 'windows'
      ? `${protocolScheme}://${protocol}.localhost/${path}`
      : `${protocol}://localhost/${path}`
  }

  window.__TAURI__.transformCallback = function transformCallback(
    callback,
    once
  ) {
    const identifier = uid()
    const prop = `_${identifier}`

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
      const callback = window.__TAURI__.transformCallback(function (r) {
        resolve(r)
        delete window[`_${error}`]
      }, true)
      const error = window.__TAURI__.transformCallback(function (e) {
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

  // open <a href="..."> links with the Tauri API
  function __openLinks() {
    document.querySelector('body').addEventListener(
      'click',
      function (e) {
        let target = e.target
        const baseTarget = document.querySelector('head base')?.target
        while (target != null) {
          if (target.matches('a')) {
            if (
              target.href &&
              (['http://', 'https://', 'mailto:', 'tel:'].some(v => target.href.startsWith(v))) &&
              (target.target === '_blank' || (!target.target &&  baseTarget === "_blank"))
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

  //-----------------------//
  // data-tauri-drag-region
  //
  // drag on mousedown and maximize on double click on Windows and Linux
  // while macOS macos maximization should be on mouseup and if the mouse
  // moves after the double click, it should be cancelled
  //-----------------------//
  const TAURI_DRAG_REGION_ATTR = 'data-tauri-drag-region';
  let x = 0, y = 0;
  document.addEventListener('mousedown', (e) => {
    if (
      // element has the magic data attribute
      e.target.hasAttribute(TAURI_DRAG_REGION_ATTR) &&
      // and was left mouse button
      e.button === 0 &&
      // and was normal click to drag or double click to maximize
      (e.detail === 1 || e.detail === 2)
      ) {

      // macOS maximization happens on `mouseup`,
      // so we save needed state and early return
      if (osName === 'macos' && e.detail == 2) {
        x = e.clientX
        y = e.clientY
        return
      }

      // prevents text cursor
      e.preventDefault()

      // fix #2549: double click on drag region edge causes content to maximize without window sizing change
      // https://github.com/tauri-apps/tauri/issues/2549#issuecomment-1250036908
      e.stopImmediatePropagation()

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
  // on macOS we maximze on mouseup instead, to match the system behavior where maximization can be canceled
  // if the mouse moves outside the data-tauri-drag-region
  if (osName === "macos") {
    document.addEventListener('mouseup', (e) => {
      if (
        // element has the magic data attribute
        e.target.hasAttribute(TAURI_DRAG_REGION_ATTR) &&
        // and was left mouse button
        e.button === 0 &&
        // and was double click
        e.detail === 2 &&
        // and the cursor hasn't moved from initial mousedown
        e.clientX === x && e.clientY === y
      ) {
        window.__TAURI_INVOKE__('tauri', {
          __tauriModule: 'Window',
          message: {
            cmd: 'manage',
            data: {
              cmd: {
                type: '__toggleMaximize'
              }
            }
          }
        })
      }
    })
  }

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
    const opts = options || {}
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
        message: message.toString()
      }
    })
  }

  window.confirm = function (message) {
    return window.__TAURI_INVOKE__('tauri', {
      __tauriModule: 'Dialog',
      message: {
        cmd: 'confirmDialog',
        message: message.toString()
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
