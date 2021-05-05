// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Send notifications to your user. Can also be used with the Notification Web API.
 * @packageDocumentation
 */

import { invokeTauriCommand } from './helpers/tauri'

/**
 * Options to send a notification.
 */
export interface Options {
  /** Notification title. */
  title: string
  /** Optional notification body. */
  body?: string
  /** Optional notification icon. */
  icon?: string
}

/** Possible permission values. */
export type Permission = 'granted' | 'denied' | 'default'

/**
 * Checks if the permission to send notifications is granted.
 *
 * @returns
 */
async function isPermissionGranted(): Promise<boolean | null> {
  if (window.Notification.permission !== 'default') {
    return Promise.resolve(window.Notification.permission === 'granted')
  }
  return invokeTauriCommand({
    __tauriModule: 'Notification',
    message: {
      cmd: 'isNotificationPermissionGranted'
    }
  })
}

/**
 * Requests the permission to send notifications.
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 */
async function requestPermission(): Promise<Permission> {
  return window.Notification.requestPermission()
}

/**
 * Sends a notification to the user.
 *
 * @param options Notification options.
 */
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
