// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Native system dialogs for opening and saving files.
 * @packageDocumentation
 */

import { invokeTauriCommand } from './helpers/tauri'

/** Extension filters for the file dialog. */
export interface DialogFilter {
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
export interface OpenDialogOptions {
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /** Initial directory or file path. It must exist. */
  defaultPath?: string
  /** Whether the dialog allows multiple selection or not. */
  multiple?: boolean
  /** Whether the dialog is a directory selection or not. */
  directory?: boolean
}

/** Options for the save dialog. */
export interface SaveDialogOptions {
  /** The filters of the dialog. */
  filters?: DialogFilter[]
  /** Initial directory or file path. It must exist. */
  defaultPath?: string
}

/**
 * Open a file/directory selection dialog
 *
 * @returns A promise resolving to the selected path(s)
 */
async function open(
  options: OpenDialogOptions = {}
): Promise<string | string[]> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return invokeTauriCommand<string | string[]>({
    __tauriModule: 'Dialog',
    mainThread: true,
    message: {
      cmd: 'openDialog',
      options
    }
  })
}

/**
 * Open a file/directory save dialog.
 *
 * @returns A promise resolving to the selected path.
 */
async function save(options: SaveDialogOptions = {}): Promise<string> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return invokeTauriCommand<string>({
    __tauriModule: 'Dialog',
    mainThread: true,
    message: {
      cmd: 'saveDialog',
      options
    }
  })
}

export { open, save }
