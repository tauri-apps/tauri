import { OpenDialogOptions, SaveDialogOptions } from './types/dialog'
import { promisified } from './tauri'

/**
 * @name openDialog
 * @description Open a file/directory selection dialog
 * @param [options]
 * @param [options.filter]
 * @param [options.defaultPath]
 * @param [options.multiple=false]
 * @param [options.directory=false]
 * @returns promise resolving to the select path(s)
 */
async function open(options: OpenDialogOptions = {}): Promise<String | String[]> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return await promisified({
    cmd: 'openDialog',
    options
  })
}

/**
 * @name save
 * @description Open a file/directory save dialog
 * @param [options]
 * @param [options.filter]
 * @param [options.defaultPath]
 * @returns promise resolving to the select path
 */
async function save(options: SaveDialogOptions = {}): Promise<String> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }

  return await promisified({
    cmd: 'saveDialog',
    options
  })
}

export {
  open,
  save
}
