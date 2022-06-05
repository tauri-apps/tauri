// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Native system dialogs for opening and saving files.
 *
 * This package is also accessible with `window.__TAURI__.dialog` when `tauri.conf.json > build > withGlobalTauri` is set to true.
 *
 * The APIs must be allowlisted on `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "dialog": {
 *         "all": true, // enable all dialog APIs
 *         "open": true, // enable file open API
 *         "save": true // enable file save API
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'

/** Extension filters for the file dialog. */
interface DialogFilter {
  /** Filter name. */
  name: string
  /**
   * Extensions to filter, without a `.` prefix.
   * @example
   * ```typescript
   * extensions: ['svg', 'png']
   * ```
   */
  extensions: string[]
}

/** Options for the open dialog. */
interface OpenDialogOptions {
  /** The title of the dialog window. */
  title?: string
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /** Initial directory or file path. */
  defaultPath?: string
  /** Whether the dialog allows multiple selection or not. */
  multiple?: boolean
  /** Whether the dialog is a directory selection or not. */
  directory?: boolean
  /**
   * If `directory` is true, indicates that it will be read recursively later.
   * Defines whether subdirectories will be allowed on the scope or not.
   */
  recursive?: boolean
}

/** Options for the save dialog. */
interface SaveDialogOptions {
  /** The title of the dialog window. */
  title?: string
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /**
   * Initial directory or file path.
   * If it's a directory path, the dialog interface will change to that folder.
   * If it's not an existing directory, the file name will be set to the dialog's file name input and the dialog will be set to the parent folder.
   */
  defaultPath?: string
}

interface MessageDialogOptions {
  /** The title of the dialog. Defaults to the app name. */
  title?: string
  /** The type of the dialog. Defaults to `info`. */
  type?: 'info' | 'warning' | 'error'
}

/**
 * Open a file/directory selection dialog.
 *
 * The selected paths are added to the filesystem and asset protocol allowlist scopes.
 * When security is more important than the easy of use of this API,
 * prefer writing a dedicated command instead.
 *
 * Note that the allowlist scope change is not persisted, so the values are cleared when the application is restarted.
 * You can save it to the filesystem using [tauri-plugin-persisted-scope](https://github.com/tauri-apps/tauri-plugin-persisted-scope).
 *
 * @returns A promise resolving to the selected path(s)
 */
async function open(
  options: OpenDialogOptions = {}
): Promise<null | string | string[]> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'openDialog',
      options
    }
  })
}

/**
 * Open a file/directory save dialog.
 *
 * The selected path is added to the filesystem and asset protocol allowlist scopes.
 * When security is more important than the easy of use of this API,
 * prefer writing a dedicated command instead.
 *
 * Note that the allowlist scope change is not persisted, so the values are cleared when the application is restarted.
 * You can save it to the filesystem using [tauri-plugin-persisted-scope](https://github.com/tauri-apps/tauri-plugin-persisted-scope).
 *
 * @returns A promise resolving to the selected path.
 */
async function save(options: SaveDialogOptions = {}): Promise<string> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'saveDialog',
      options
    }
  })
}

/**
 * Shows a message dialog with an `Ok` button.
 *
 * @param {string} message The message to show.
 * @param {string|MessageDialogOptions|undefined} options The dialog's options. If a string, it represents the dialog title.
 *
 * @return {Promise<void>} A promise indicating the success or failure of the operation.
 */
async function message(
  message: string,
  options?: string | MessageDialogOptions
): Promise<void> {
  const opts = typeof options === 'string' ? { title: options } : options
  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'messageDialog',
      message,
      title: opts?.title,
      type: opts?.type
    }
  })
}

/**
 * Shows a question dialog with `Yes` and `No` buttons.
 *
 * @param {string} message The message to show.
 * @param {string|MessageDialogOptions|undefined} options The dialog's options. If a string, it represents the dialog title.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Yes` was clicked or not.
 */
async function ask(
  message: string,
  options?: string | MessageDialogOptions
): Promise<boolean> {
  const opts = typeof options === 'string' ? { title: options } : options
  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'askDialog',
      message,
      title: opts?.title,
      type: opts?.type
    }
  })
}

/**
 * Shows a question dialog with `Ok` and `Cancel` buttons.
 *
 * @param {string} message The message to show.
 * @param {string|MessageDialogOptions|undefined} options The dialog's options. If a string, it represents the dialog title.
 *
 * @return {Promise<void>} A promise resolving to a boolean indicating whether `Ok` was clicked or not.
 */
async function confirm(
  message: string,
  options?: string | MessageDialogOptions
): Promise<boolean> {
  const opts = typeof options === 'string' ? { title: options } : options
  return invokeTauriCommand({
    __tauriModule: 'Dialog',
    message: {
      cmd: 'confirmDialog',
      message,
      title: opts?.title,
      type: opts?.type
    }
  })
}

export type {
  DialogFilter,
  OpenDialogOptions,
  SaveDialogOptions,
  MessageDialogOptions
}

export { open, save, message, ask, confirm }
