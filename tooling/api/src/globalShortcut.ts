// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { transformCallback } from './tauri'

export type ShortcutHandler = (shortcut: string) => void

/**
 * Register a global shortcut.
 *
 * @param shortcut Shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 * @returns
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
 *
 * @param shortcuts Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @param handler Shortcut handler callback - takes the triggered shortcut as argument
 * @returns
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
 *
 * @param shortcut Array of shortcut definitions, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @returns A promise resolving to the state.
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
 *
 * @param shortcut shortcut definition, modifiers and key separated by "+" e.g. CmdOrControl+Q
 * @returns
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
 *
 * @returns
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
