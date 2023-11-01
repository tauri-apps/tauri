// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Send toast notifications (brief auto-expiring OS window element) to your user.
 * Can also be used with the Notification Web API.
 *
 * This package is also accessible with `window.__TAURI__.notification` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.notification`](https://tauri.app/v1/api/config/#allowlistconfig.notification) in `tauri.conf.json`:
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
 *
 * @since 1.0.0
 */
interface Options {
  /** Notification title. */
  title: string
  /** Optional notification body. */
  body?: string
  /**
   * Optional notification icon.
   *
   * #### Platform-specific
   *
   * - **Windows**: The app must be installed for this to have any effect.
   *
   */
  icon?: string
  /**
   * Optional notification sound.
   *
   * #### Platform-specific
   *
   * Each OS has a different sound name so you will need to conditionally specify an appropriate sound
   * based on the OS in use, 'default' represents the default system sound. For a list of sounds see:
   * - **Linux**: can be one of the sounds listed in {@link https://0pointer.de/public/sound-naming-spec.html}
   * - **Windows**: can be one of the sounds listed in {@link https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-audio}
   *   but without the prefix, for example, if `ms-winsoundevent:Notification.Default` you would use `Default` and
   *   if `ms-winsoundevent:Notification.Looping.Alarm2`, you would use `Alarm2`.
   *   Windows 7 is not supported, if a sound is provided, it will play the default sound, otherwise it will be silent.
   * - **macOS**: you can specify the name of the sound you'd like to play when the notification is shown.
   * Any of the default sounds (under System Preferences > Sound) can be used, in addition to custom sound files.
   * Be sure that the sound file is copied under the app bundle (e.g., `YourApp.app/Contents/Resources`), or one of the following locations:
   *   - `~/Library/Sounds`
   *   - `/Library/Sounds`
   *   - `/Network/Library/Sounds`
   *   - `/System/Library/Sounds`
   *
   *   See the {@link https://developer.apple.com/documentation/appkit/nssound | NSSound} docs for more information.
   *
   * @since 1.5.0
   */
  sound?: 'default' | string
}

/** Possible permission values. */
type Permission = 'granted' | 'denied' | 'default'

/**
 * Checks if the permission to send notifications is granted.
 * @example
 * ```typescript
 * import { isPermissionGranted } from '@tauri-apps/api/notification';
 * const permissionGranted = await isPermissionGranted();
 * ```
 *
 * @since 1.0.0
 */
async function isPermissionGranted(): Promise<boolean> {
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
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission } from '@tauri-apps/api/notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * ```
 *
 * @returns A promise resolving to whether the user granted the permission or not.
 *
 * @since 1.0.0
 */
async function requestPermission(): Promise<Permission> {
  return window.Notification.requestPermission()
}

/**
 * Sends a notification to the user.
 * @example
 * ```typescript
 * import { isPermissionGranted, requestPermission, sendNotification } from '@tauri-apps/api/notification';
 * let permissionGranted = await isPermissionGranted();
 * if (!permissionGranted) {
 *   const permission = await requestPermission();
 *   permissionGranted = permission === 'granted';
 * }
 * if (permissionGranted) {
 *   sendNotification('Tauri is awesome!');
 *   sendNotification({ title: 'TAURI', body: 'Tauri is awesome!' });
 * }
 * ```
 *
 * @since 1.0.0
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
