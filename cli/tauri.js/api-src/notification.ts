import { promisified } from './tauri'

export interface Options {
  title: string
  body?: string
  icon?: string
}

export type PartialOptions = Omit<Options, 'title'>
export type Permission = 'granted' | 'denied' | 'default'

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

export { sendNotification, requestPermission, isPermissionGranted }
