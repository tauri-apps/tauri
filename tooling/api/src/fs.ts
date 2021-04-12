// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'

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
  Current
}

export interface FsOptions {
  dir?: BaseDirectory
}

export interface FsDirOptions {
  dir?: BaseDirectory
  recursive?: boolean
}

export interface FsTextFileOption {
  path: string
  contents: string
}

export interface FsBinaryFileOption {
  path: string
  contents: ArrayBuffer
}

export interface FileEntry {
  path: string
  /**
   * Name of the directory/file
   * can be null if the path terminates with `..`
   */
  name?: string
  /** Children of this entry if it's a directory; null otherwise */
  children?: FileEntry[]
}

/**
 * Reads a file as text.
 *
 * @param filePath Path to the file
 * @param [options]
 * @returns A promise resolving to a string of the file content.
 */
async function readTextFile(
  filePath: string,
  options: FsOptions = {}
): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'readTextFile',
      path: filePath,
      options
    }
  })
}

/**
 * Reads a file as binary.
 *
 * @param filePath Path to the file
 * @param [options]
 * @returns A promise resolving to an array of the file bytes.
 */
async function readBinaryFile(
  filePath: string,
  options: FsOptions = {}
): Promise<number[]> {
  return invokeTauriCommand<number[]>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'readBinaryFile',
      path: filePath,
      options
    }
  })
}

/**
 * Writes a text file.
 *
 * @param file File configuration object
 * @param [options]
 * @returns
 */
async function writeFile(
  file: FsTextFileOption,
  options: FsOptions = {}
): Promise<void> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }
  if (typeof file === 'object') {
    Object.freeze(file)
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'writeFile',
      path: file.path,
      contents: file.contents,
      options
    }
  })
}

const CHUNK_SIZE = 65536

/**
 * Convert an Uint8Array to ascii string.
 *
 * @param arr
 * @returns An ASCII string.
 */
function uint8ArrayToString(arr: Uint8Array): string {
  if (arr.length < CHUNK_SIZE) {
    return String.fromCharCode.apply(null, Array.from(arr))
  }

  let result = ''
  const arrLen = arr.length
  for (let i = 0; i < arrLen; i++) {
    const chunk = arr.subarray(i * CHUNK_SIZE, (i + 1) * CHUNK_SIZE)
    result += String.fromCharCode.apply(null, Array.from(chunk))
  }
  return result
}

/**
 * Convert an ArrayBuffer to base64 encoded string.
 *
 * @param buffer
 * @returns A base64 encoded string.
 */
function arrayBufferToBase64(buffer: ArrayBuffer): string {
  const str = uint8ArrayToString(new Uint8Array(buffer))
  return btoa(str)
}

/**
 * Writes a binary file
 *
 * @param file File configuration object
 * @param [options]
 * @returns
 */
async function writeBinaryFile(
  file: FsBinaryFileOption,
  options: FsOptions = {}
): Promise<void> {
  if (typeof options === 'object') {
    Object.freeze(options)
  }
  if (typeof file === 'object') {
    Object.freeze(file)
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'writeBinaryFile',
      path: file.path,
      contents: arrayBufferToBase64(file.contents),
      options
    }
  })
}

/**
 * List directory files.
 *
 * @param dir Path to the directory to read
 * @param [options] Configuration object
 * @returns
 */
async function readDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<FileEntry[]> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'readDir',
      path: dir,
      options
    }
  })
}

/**
 * Creates a directory.
 * If one of the path's parent components doesn't exist
 * and the `recursive` option isn't set to true, it will be rejected.
 *
 * @param dir Path to the directory to create
 * @param [options] Configuration object
 * @returns
 */
async function createDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'createDir',
      path: dir,
      options
    }
  })
}

/**
 * Removes a directory.
 * If the directory is not empty and the `recursive` option isn't set to true, it will be rejected.
 *
 * @param dir Path to the directory to remove
 * @param [options] Configuration object
 * @returns
 */
async function removeDir(
  dir: string,
  options: FsDirOptions = {}
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'removeDir',
      path: dir,
      options
    }
  })
}

/**
 * Copys a file to a destination.
 *
 * @param source A path of the file to copy
 * @param destination A path for the destination file
 * @param [options] Configuration object
 * @returns
 */
async function copyFile(
  source: string,
  destination: string,
  options: FsOptions = {}
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'copyFile',
      source,
      destination,
      options
    }
  })
}

/**
 * Removes a file.
 *
 * @param file Path to the file to remove
 * @param [options] Configuration object
 * @returns
 */
async function removeFile(
  file: string,
  options: FsOptions = {}
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'removeFile',
      path: file,
      options: options
    }
  })
}

/**
 * Renames a file
 *
 * @param oldPath A path of the file to rename
 * @param newPath A path of the new file name
 * @param [options] Configuration object
 * @returns
 */
async function renameFile(
  oldPath: string,
  newPath: string,
  options: FsOptions = {}
): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'renameFile',
      oldPath,
      newPath,
      options
    }
  })
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
  renameFile
}
