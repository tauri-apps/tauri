// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'

export interface DialogFilter {
  name: string
  extensions: string[]
}

export interface OpenDialogOptions {
  filters?: DialogFilter[]
  defaultPath?: string
  multiple?: boolean
  directory?: boolean
}

export interface SaveDialogOptions {
  filters?: DialogFilter[]
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
