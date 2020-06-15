import tauri from './tauri'

/**
 * @name openDialog
 * @description Open a file/directory selection dialog
 * @param {Object} [options]
 * @param {String} [options.filter]
 * @param {String} [options.defaultPath]
 * @param {Boolean} [options.multiple=false]
 * @param {Boolean} [options.directory=false]
 * @returns {Promise<String|String[]>} promise resolving to the select path(s)
 */
function open (options = {}) {
  return tauri.openDialog(options)
}

/**
 * @name save
 * @description Open a file/directory save dialog
 * @param {Object} [options]
 * @param {String} [options.filter]
 * @param {String} [options.defaultPath]
 * @returns {Promise<String>} promise resolving to the select path
 */
function save (options = {}) {
  return tauri.saveDialog(options)
}

export {
  open,
  save
}
