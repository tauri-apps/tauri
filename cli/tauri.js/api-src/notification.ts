import { promisified } from './tauri'

var __permissionSettable = false
var __nermission = 'default'
function __setNotificationPermission(value: 'granted' | 'denied' | 'default') {
  __permissionSettable = true
  // @ts-ignore
  window.Notification.permission = value
  __permissionSettable = false
}

// @ts-ignore
window.Notification = function (title, options) {
  if (options === void 0) {
    options = {}
  }
  options.title = title
  sendNotification(options)
}
window.Notification.requestPermission = requestPermission

Object.defineProperty(window.Notification, 'permission', {
  enumerable: true,
  get: function () {
    return __nermission
  },
  set: function (v) {
    if (!__permissionSettable) {
      throw new Error('Readonly property')
    }
    __nermission = v
  }
})

isPermissionGranted()
  .then(function (response) {
    if (response === null) {
      __setNotificationPermission('default')
    } else {
      __setNotificationPermission(response ? 'granted' : 'denied')
    }
  })

function isPermissionGranted() {
  if (window.Notification.permission !== 'default') {
    return Promise.resolve(window.Notification.permission === 'granted')
  }
  return promisified({
    cmd: 'isNotificationPermissionGranted'
  })
}

function requestPermission() {
  return promisified({
    cmd: 'requestNotificationPermission'
  }).then(function (state) {
    __setNotificationPermission(state)
    return state
  })
}

function sendNotification(options) {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return isPermissionGranted()
    .then(function (permission) {
      if (permission) {
        return promisified({
          cmd: 'notification',
          options: typeof options === 'string' ? {
            body: options
          } : options
        });
      }
    })
}

export {
  sendNotification,
  isPermissionGranted
}
