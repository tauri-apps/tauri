import { promisified } from './tauri'

export interface OpenDialogOptions {
  filter?: string
  defaultPath?: string
  multiple?: boolean
  directory?: boolean
}

export type SaveDialogOptions = Pick<
  OpenDialogOptions,
  'filter' | 'defaultPath'
>

/**
 * @name openDialog
 * @description Open a file/directory selection dialog
 * @param {Object} [options]
 * @param {string} [options.filter]
 * @param {string} [options.defaultPath]
 * @param {boolean} [options.multiple=false]
 * @param {boolean} [options.directory=false]
 * @returns {Promise<string | string[]>} Promise resolving to the select path(s)
 */
async function open(
  options: OpenDialogOptions = {}
): Promise<string | string[]> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return await promisified({
    module: 'Dialog',
    message: {
      cmd: 'openDialog',
      options
    }
  })
}

/**
 * @name save
 * @description Open a file/directory save dialog
 * @param {Object} [options]
 * @param {string} [options.filter]
 * @param {string} [options.defaultPath]
 * @returns {Promise<string>} Promise resolving to the select path
 */
async function save(options: SaveDialogOptions = {}): Promise<string> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return await promisified({
    module: 'Dialog',
    message: {
      cmd: 'saveDialog',
      options
    }
  })
}

export { open, save }
