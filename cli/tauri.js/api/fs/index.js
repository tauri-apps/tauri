import tauri from './tauri'
import { Dir } from './dir'

/**
 * reads a file as text
 *
 * @param {String} filePath path to the file
 * @param {Object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<string>}
 */
function readTextFile (filePath, options = {}) {
  return tauri.readTextFile(filePath, options)
}

/**
 * reads a file as binary
 *
 * @param {String} filePath path to the file
 * @param {Object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<int[]>}
 */
function readBinaryFile (filePath, options = {}) {
  return tauri.readBinaryFile(filePath, options)
}

/**
 * writes a text file
 *
 * @param {Object} file
 * @param {String} file.path path of the file
 * @param {String} file.contents contents of the file
 * @param {Object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function writeFile (file, options = {}) {
  return tauri.writeFile(file, options)
}

/**
 * @typedef {Object} FileEntry
 * @property {String} path
 * @property {Boolean} is_dir
 * @property {String} name
 */

/**
 * list directory files
 *
 * @param {String} dir path to the directory to read
 * @param {Object} [options] configuration object
 * @param {Boolean} [options.recursive] whether to list dirs recursively or not
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<FileEntry[]>}
 */
function readDir (dir, options = {}) {
  return tauri.readDir(dir, options)
}

/**
 * Creates a directory
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, it will be rejected
 *
 * @param {String} dir path to the directory to create
 * @param {Object} [options] configuration object
 * @param {Boolean} [options.recursive] whether to create the directory's parent components or not
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function createDir (dir, options = {}) {
  return tauri.createDir(dir, options)
}

/**
 * Removes a directory
 * If the directory is not empty and the `recursive` option isn't set to true, it will be rejected
 *
 * @param {String} dir path to the directory to remove
 * @param {Object} [options] configuration object
 * @param {Boolean} [options.recursive] whether to remove all of the directory's content or not
 * @param {BaseDirectory} [options.dir] base directory
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
 * @param {object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function copyFile (source, destination, options = {}) {
  return tauri.copyFile(source, destination, options)
}

/**
 * Removes a file
 *
 * @param {String} file path to the file to remove
 * @param {Object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function removeFile (file, options = {}) {
  return tauri.removeFile(file, options)
}

/**
 * Renames a file
 *
 * @param {String} oldPath
 * @param {String} newPath
 * @param {Object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function renameFile (oldPath, newPath, options = {}) {
  return tauri.renameFile(oldPath, newPath, options)
}

export {
  Dir,
  readTextFile,
  readBinaryFile,
  writeFile,
  readDir,
  createDir,
  removeDir,
  copyFile,
  removeFile,
  renameFile
}
