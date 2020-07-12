import { Options, Permission } from './types/notification'
import { promisified } from './tauri'

async function isPermissionGranted(): Promise<boolean | null> {
  if (window.Notification.permission !== 'default') {
    return await Promise.resolve(window.Notification.permission === 'granted')
  }
  return await promisified({
    cmd: 'isNotificationPermissionGranted'
  })
}

async function requestPermission(): Promise<Permission> {
  return await window.Notification.requestPermission()
}

function sendNotification(options: Options | string): void {
  if (typeof options === 'string') {
    // eslint-disable-next-line no-new
    new window.Notification(options)
  } else {
    // eslint-disable-next-line no-new
    new window.Notification(options.title, options)
  }
}

export {
  sendNotification,
  requestPermission,
  isPermissionGranted
}
