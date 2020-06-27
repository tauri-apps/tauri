import { Options, PartialOptions, Permission } from './types/notification'
import { promisified } from './tauri'

let permissionSettable = false
let permissionValue = 'default'
function setNotificationPermission(value: Permission): void {
  permissionSettable = true
  // @ts-expect-error
  window.Notification.permission = value
  permissionSettable = false
}

// @ts-expect-error
window.Notification = (title: string, options?: PartialOptions) => {
  sendNotification({
    title,
    ...options
  })
}

window.Notification.requestPermission = requestPermission

Object.defineProperty(window.Notification, 'permission', {
  enumerable: true,
  get: () => permissionValue,
  set(v: Permission) {
    if (!permissionSettable) {
      throw new Error('Readonly property')
    }
    permissionValue = v
  }
})

isPermissionGranted()
  .then(response => {
    if (response === null) {
      setNotificationPermission('default')
    } else {
      setNotificationPermission(response ? 'granted' : 'denied')
    }
  })
  .catch(err => { throw err })

async function isPermissionGranted(): Promise<boolean | null> {
  if (window.Notification.permission !== 'default') {
    return await Promise.resolve(window.Notification.permission === 'granted')
  }
  return await promisified({
    cmd: 'isNotificationPermissionGranted'
  })
}

async function requestPermission(): Promise<Permission> {
  return await promisified<Permission>({
    cmd: 'requestNotificationPermission'
  }).then(permission => {
    setNotificationPermission(permission)
    return permission
  })
}

function sendNotification(options: Options | string): void {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  isPermissionGranted()
    .then(permission => {
      if (permission) {
        return promisified({
          cmd: 'notification',
          options: typeof options === 'string' ? {
            body: options
          } : options
        })
      }
    })
    .catch(err => { throw err })
}

export {
  sendNotification,
  isPermissionGranted
}
