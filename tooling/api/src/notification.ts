// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used with the Notification Web API.
 *
 * This package is also accessible with `window.__TAURI__.notification` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "notification": {
 *         "all": true // enable all notification APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

/**
 * Options to send a notification.
 */
interface Options {
  /** Notification title. */
  title: string
  /** Optional notification body. */
  body?: string
  /** Optional notification icon. */
  icon?: string
}

/** Possible permission values. */
type Permission = 'granted' | 'denied' | 'default'

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

export type { Options, Permission }

export { sendNotification, requestPermission, isPermissionGranted }
