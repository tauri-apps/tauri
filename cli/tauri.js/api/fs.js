import tauri from './tauri'

/**
 * reads a file as text
 *
 * @param {string} filePath path to the file
 * @return {Promise<string>}
 */
function readTextFile (filePath) {
  return tauri.readTextFile(filePath)
}

/**
 * reads a file as binary
 *
 * @param {string} filePath path to the file
 * @return {Promise<int[]>}
 */
function readBinaryFile (filePath) {
  return tauri.readBinaryFile(filePath)
}

/**
 * writes a text file
 *
 * @param {object} file
 * @param {string} file.path path of the file
 * @param {string} file.contents contents of the file
 * @return {Promise<void>}
 */
function writeFile (file) {
  return tauri.writeFile(file)
}

/**
 * @typedef {object} FileEntry
 * @property {string} path
 * @property {boolean} is_dir
 * @property {string} name
 */

/**
 * list directory files
 *
 * @param {string} dir path to the directory to read
 * @param {object} [options] configuration object
 * @param {boolean} [options.recursive] whether to list dirs recursively or not
 * @return {Promise<FileEntry[]>}
 */
function readDir (dir, options = {}) {
  return tauri.readDir(dir, options)
}

/**
 * Creates a directory
 * If one of the path's parent components doesn't exist and the `recursive` option isn't set to true, it will be rejected
 *
 * @param {string} dir path to the directory to create
 * @param {object} [options] configuration object
 * @param {boolean} [options.recursive] whether to create the directory's parent components or not
 * @return {Promise<void>}
 */
function createDir (dir, options = {}) {
  return tauri.createDir(dir, options)
}

/**
 * Removes a directory
 * If the directory is not empty and the `recursive` option isn't set to true, it will be rejected
 *
 * @param {string} dir path to the directory to remove
 * @param {object} [options] configuration object
 * @param {boolean} [options.recursive] whether to remove all of the directory's content or not
 * @return {Promise<void>}
 */
function removeDir (dir, options = {}) {
  return tauri.removeDir(dir, options)
}

/**
 * Copy file
 *
 * @param {string} source
 * @param {string} destination
 * @return {Promise<void>}
 */
function copyFile (source, destination) {
  return tauri.copyFile(source, destination)
}

/**
 * Removes a file
 *
 * @param {string} file path to the file to remove
 * @return {Promise<void>}
 */
function removeFile (file) {
  return tauri.removeFile(file)
}

/**
 * Renames a file
 *
 * @param {string} oldPath
 * @param {string} newPath
 * @return {Promise<void>}
 */
function renameFile (oldPath, newPath) {
  return tauri.renameFile(oldPath, newPath)
}

export {
  readTextFile,
  readBinaryFile,
  writeFile,
  readDir,
  createDir,
  removeDir,
  copyFile,
  removeFile
}
