import { OpenDialogOptions, SaveDialogOptions } from './models'
import tauri from './tauri'

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
  return tauri.openDialog(options)
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
  return tauri.saveDialog(options)
}

export {
  open,
  save
}
