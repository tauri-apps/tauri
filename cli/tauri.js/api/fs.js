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

export {
  readTextFile,
  readBinaryFile,
  writeFile,
  readDir
}
