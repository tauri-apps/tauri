import tauri from './tauri'
import { Dir } from './dir'

/**
 * reads a file as text
 *
 * @param {string} filePath path to the file
 * @param {object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<string>}
 */
function readTextFile (filePath, options = {}) {
  return tauri.readTextFile(filePath, options)
}

/**
 * reads a file as binary
 *
 * @param {string} filePath path to the file
 * @param {object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<int[]>}
 */
function readBinaryFile (filePath, options = {}) {
  return tauri.readBinaryFile(filePath, options)
}

/**
 * writes a text file
 *
 * @param {object} file
 * @param {string} file.path path of the file
 * @param {string} file.contents contents of the file
 * @param {object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function writeFile (file, options = {}) {
  return tauri.writeFile(file, options)
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
 * @param {string} dir path to the directory to create
 * @param {object} [options] configuration object
 * @param {boolean} [options.recursive] whether to create the directory's parent components or not
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
 * @param {string} dir path to the directory to remove
 * @param {object} [options] configuration object
 * @param {boolean} [options.recursive] whether to remove all of the directory's content or not
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
 * @param {string} file path to the file to remove
 * @param {object} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<void>}
 */
function removeFile (file, options = {}) {
  return tauri.removeFile(file, options)
}

/**
 * Renames a file
 *
 * @param {string} oldPath
 * @param {string} newPath
 * @param {object} [options] configuration object
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
