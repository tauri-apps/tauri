import { promisified } from "./tauri";

export enum BaseDirectory {
  Audio = 1,
  Cache,
  Config,
  Data,
  LocalData,
  Desktop,
  Document,
  Download,
  Executable,
  Font,
  Home,
  Picture,
  Public,
  Runtime,
  Template,
  Video,
  Resource,
  App,
}

export interface FsOptions {
  dir?: BaseDirectory;
}

export interface FsDirOptions {
  dir?: BaseDirectory;
  recursive?: boolean;
}

export interface FsTextFileOption {
  path: string;
  contents: string;
}

export interface FsBinaryFileOption {
  path: string;
  contents: ArrayBuffer;
}

export interface FileEntry {
  path: string;
  // name of the directory/file
  // can be null if the path terminates with `..`
  name?: string;
  // children of this entry if it's a directory; null otherwise
  children?: FileEntry[];
}

/**
 * @name readTextFile
 * @description Reads a file as text
 * @param {string} filePath path to the file
 * @param {FsOptions} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<string>}
 */
async function readTextFile(
  filePath: string,
  options: FsOptions = {}
): Promise<string> {
  return await promisified<string>({
    cmd: "readTextFile",
    path: filePath,
    options,
  });
}

/**
 * @name readBinaryFile
 * @description Reads a file as binary
 * @param {string} filePath path to the file
 * @param {FsOptions} [options] configuration object
 * @param {BaseDirectory} [options.dir] base directory
 * @return {Promise<number[]>}
 */
async function readBinaryFile(
  filePath: string,
  options: FsOptions = {}
): Promise<number[]> {
  return await promisified<number[]>({
    cmd: "readBinaryFile",
    path: filePath,
    options,
  });
}

/**
 * writes a text file
 *
 * @param file
 * @param file.path path of the file
 * @param file.contents contents of the file
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function writeFile(
  file: FsTextFileOption,
  options: FsOptions = {}
): Promise<void> {
  if (typeof options === "object") {
    Object.freeze(options);
  }
  if (typeof file === "object") {
    Object.freeze(file);
  }

  return await promisified({
    cmd: "writeFile",
    path: file.path,
    contents: file.contents,
    options,
  });
}

const CHUNK_SIZE = 65536;

/**
 * convert an Uint8Array to ascii string
 *
 * @param arr
 * @return ASCII string
 */
function uint8ArrayToString(arr: Uint8Array): string {
  if (arr.length < CHUNK_SIZE) {
    return String.fromCharCode.apply(null, Array.from(arr));
  }

  let result = "";
  const arrLen = arr.length;
  for (let i = 0; i < arrLen; i++) {
    const chunk = arr.subarray(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE);
    result += String.fromCharCode.apply(null, Array.from(chunk));
  }
  return result;
}

/**
 * convert an ArrayBuffer to base64 encoded string
 *
 * @param buffer
 * @return base64 encoded string
 */
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const str = uint8ArrayToString(new Uint8Array(buffer));
  return btoa(str);
}

/**
 * writes a binary file
 *
 * @param file
 * @param file.path path of the file
 * @param file.contents contents of the file
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function writeBinaryFile(
  file: FsBinaryFileOption,
  options: FsOptions = {}
): Promise<void> {
  if (typeof options === "object") {
    Object.freeze(options);
  }
  if (typeof file === "object") {
    Object.freeze(file);
  }

  return await promisified({
    cmd: "writeBinaryFile",
    path: file.path,
    contents: arrayBufferToBase64(file.contents),
    options,
  });
}

/**
 * list directory files
 *
 * @param dir path to the directory to read
 * @param [options] configuration object
 * @param [options.recursive] whether to list dirs recursively or not
 * @param [options.dir] base directory
 * @return
 */
async function readDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<FileEntry[]> {
  return await promisified({
    cmd: "readDir",
    path: dir,
    options,
  });
}

/**
 * Creates a directory
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, it will be rejected
 *
 * @param dir path to the directory to create
 * @param [options] configuration object
 * @param [options.recursive] whether to create the directory's parent components or not
 * @param [options.dir] base directory
 * @return
 */
async function createDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<void> {
  return await promisified({
    cmd: "createDir",
    path: dir,
    options,
  });
}

/**
 * Removes a directory
 * If the directory is not empty and the `recursive` option isn't set to true, it will be rejected
 *
 * @param dir path to the directory to remove
 * @param [options] configuration object
 * @param [options.recursive] whether to remove all of the directory's content or not
 * @param [options.dir] base directory
 * @return
 */
async function removeDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<void> {
  return await promisified({
    cmd: "removeDir",
    path: dir,
    options,
  });
}

/**
 * Copy file
 *
 * @param source
 * @param destination
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function copyFile(
  source: string,
  destination: string,
  options: FsOptions = {}
): Promise<void> {
  return await promisified({
    cmd: "copyFile",
    source,
    destination,
    options,
  });
}

/**
 * Removes a file
 *
 * @param file path to the file to remove
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function removeFile(
  file: string,
  options: FsOptions = {}
): Promise<void> {
  return await promisified({
    cmd: "removeFile",
    path: file,
    options: options,
  });
}

/**
 * Renames a file
 *
 * @param oldPath
 * @param newPath
 * @param [options] configuration object
 * @param [options.dir] base directory
 * @return
 */
async function renameFile(
  oldPath: string,
  newPath: string,
  options: FsOptions = {}
): Promise<void> {
  return await promisified({
    cmd: "renameFile",
    oldPath,
    newPath,
    options,
  });
}

export {
  BaseDirectory as Dir,
  readTextFile,
  readBinaryFile,
  writeFile,
  writeBinaryFile,
  readDir,
  createDir,
  removeDir,
  copyFile,
  removeFile,
  renameFile,
};
