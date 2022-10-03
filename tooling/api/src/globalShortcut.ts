// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Register global shortcuts.
 *
 * This package is also accessible with `window.__TAURI__.globalShortcut` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.globalShortcut`](https://tauri.app/v1/api/config/#allowlistconfig.globalshortcut) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "globalShortcut": {
 *         "all": true // enable all global shortcut APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

export type ShortcutHandler = (shortcut: string) => void

/**
 * Register a global shortcut.
 * @example
 * ```typescript
 * import { register } from '@tauri-apps/api/globalShortcut';
 * await register('CommandOrControl+Shift+C', () => {
 *   console.log('Shortcut triggered');
 * });
 * ```
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 1.0.0
 */
async function register(
  shortcut: string,
  handler: ShortcutHandler
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'register',
      shortcut,
      handler: transformCallback(handler)
    }
  })
}

/**
 * Register a collection of global shortcuts.
 * @example
 * ```typescript
 * import { registerAll } from '@tauri-apps/api/globalShortcut';
 * await registerAll(['CommandOrControl+Shift+C', 'Ctrl+Alt+F12'], (shortcut) => {
 *   console.log(`Shortcut ${shortcut} triggered`);
 * });
 * ```
 *
 * @param shortcuts Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 *
 * @since 1.0.0
 */
async function registerAll(
  shortcuts: string[],
  handler: ShortcutHandler
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'registerAll',
      shortcuts,
      handler: transformCallback(handler)
    }
  })
}

/**
 * Determines whether the given shortcut is registered by this application or not.
 * @example
 * ```typescript
 * import { isRegistered } from '@tauri-apps/api/globalShortcut';
 * const isRegistered = await isRegistered('CommandOrControl+P');
 * ```
 *
 * @param shortcut Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 1.0.0
 */
async function isRegistered(shortcut: string): Promise<boolean> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'isRegistered',
      shortcut
    }
  })
}

/**
 * Unregister a global shortcut.
 * @example
 * ```typescript
 * import { unregister } from '@tauri-apps/api/globalShortcut';
 * await unregister('CmdOrControl+Space');
 * ```
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 *
 * @since 1.0.0
 */
async function unregister(shortcut: string): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregister',
      shortcut
    }
  })
}

/**
 * Unregisters all shortcuts registered by the application.
 * @example
 * ```typescript
 * import { unregisterAll } from '@tauri-apps/api/globalShortcut';
 * await unregisterAll();
 * ```
 *
 * @since 1.0.0
 */
async function unregisterAll(): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'GlobalShortcut',
    message: {
      cmd: 'unregisterAll'
    }
  })
}

export { register, registerAll, isRegistered, unregister, unregisterAll }
