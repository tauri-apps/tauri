// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * Access the file system.
 *
 * This package is also accessible with `window.__TAURI__.fs` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.fs`](https://tauri.app/v1/api/config/#allowlistconfig.fs) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "fs": {
 *         "all": true, // enable all FS APIs
 *         "readFile": true,
 *         "writeFile": true,
 *         "readDir": true,
 *         "copyFile": true,
 *         "createDir": true,
 *         "removeDir": true,
 *         "removeFile": true,
 *         "renameFile": true
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 *
 * ## Security
 *
 * This module prevents path traversal, not allowing absolute paths or parent dir components
 * (i.e. "/usr/path/to/file" or "../path/to/file" paths are not allowed).
 * Paths accessed with this API must be relative to one of the {@link BaseDirectory | base directories}
 * so if you need access to arbitrary filesystem paths, you must write such logic on the core layer instead.
 *
 * The API has a scope configuration that forces you to restrict the paths that can be accessed using glob patterns.
 *
 * The scope configuration is an array of glob patterns describing folder paths that are allowed.
 * For instance, this scope configuration only allows accessing files on the
 * *databases* folder of the {@link path.appDir | $APP directory}:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "fs": {
 *         "scope": ["$APP/databases/*"]
 *       }
 *     }
 *   }
 * }
 * ```
 *
 * Notice the use of the `$APP` variable. The value is injected at runtime, resolving to the {@link path.appDir | app directory}.
 * The available variables are:
 * {@link path.audioDir | `$AUDIO`}, {@link path.cacheDir | `$CACHE`}, {@link path.configDir | `$CONFIG`}, {@link path.dataDir | `$DATA`},
 * {@link path.localDataDir | `$LOCALDATA`}, {@link path.desktopDir | `$DESKTOP`}, {@link path.documentDir | `$DOCUMENT`},
 * {@link path.downloadDir | `$DOWNLOAD`}, {@link path.executableDir | `$EXE`}, {@link path.fontDir | `$FONT`}, {@link path.homeDir | `$HOME`},
 * {@link path.pictureDir | `$PICTURE`}, {@link path.publicDir | `$PUBLIC`}, {@link path.runtimeDir | `$RUNTIME`},
 * {@link path.templateDir | `$TEMPLATE`}, {@link path.videoDir | `$VIDEO`}, {@link path.resourceDir | `$RESOURCE`}, {@link path.appDir | `$APP`},
 * {@link path.logDir | `$LOG`}, {@link os.tempdir | `$TEMP`}.
 *
 * Trying to execute any API with a URL not configured on the scope results in a promise rejection due to denied access.
 *
 * Note that this scope applies to **all** APIs on this module.
 *
 * @module
 */

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
  Log,
  Temp
}

interface FsOptions {
  dir?: BaseDirectory
  // note that adding fields here needs a change in the writeBinaryFile check
}

interface FsDirOptions {
  dir?: BaseDirectory
  recursive?: boolean
}

/** Options object used to write a UTF-8 string to a file. */
interface FsTextFileOption {
  /** Path to the file to write. */
  path: string
  /** The UTF-8 string to write to the file. */
  contents: string
}

type BinaryFileContents = Iterable<number> | ArrayLike<number> | ArrayBuffer

/** Options object used to write a binary data to a file. */
interface FsBinaryFileOption {
  /** Path to the file to write. */
  path: string
  /** The byte array contents. */
  contents: BinaryFileContents
}

interface FileEntry {
  path: string
  /**
   * Name of the directory/file
   * can be null if the path terminates with `..`
   */
  name?: string
  /** Children of this entry if it's a directory; null otherwise */
  children?: FileEntry[]
}

export interface ReadFileOptions {
  /** Base directory for `path` */
  baseDir?: BaseDirectory
}

/**
 * Reads and resolves to the entire contents of a file as an array of bytes. TextDecoder can be used to transform the bytes to string if required.
 * @example
 * ```typescript
 * import { readFile, BaseDirectory } from '@tauri-apps/api/fs';
 * const contents = await readFile('avatar.png', { dir: BaseDirectory.Resource });
 * ```
 */
async function readFile(
  path: string | URL,
  options?: ReadFileOptions
): Promise<Uint8Array> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  const arr = await invokeTauriCommand<number[]>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'readFile',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })

  return Uint8Array.from(arr)
}

/**
 * Reads and returns the entire contents of a file as UTF-8 string.
 * @example
 * ```typescript
 * import { readTextFile, BaseDirectory } from '@tauri-apps/api/fs';
 * const contents = await readTextFile('app.conf', { dir: BaseDirectory.App });
 * ```
 */
async function readTextFile(
  path: string | URL,
  options?: ReadFileOptions
): Promise<string> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'readTextFile',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })
}

export interface WriteFileOptions {
  /** Defaults to false. If set to true, will append to a file instead of overwriting previous contents. */
  append?: boolean
  /** Sets the option to allow creating a new file, if one doesn't already exist at the specified path (defaults to true). */
  create?: boolean
  /** Unix file permissions. */
  mode?: number
  /**
   * Windows file permissions.
   * Overrides the `dwDesiredAccess` argument to the call to [`CreateFile`]https://docs.microsoft.com/en-us/windows/win32/api/fileapi/nf-fileapi-createfilea with the specified value.
   */
  accessMode?: number
  /** Base directory for `path` */
  baseDir?: BaseDirectory
}

/**
 * Write `data` to the given `path`, by default creating a new file if needed, else overwriting.
 * @example
 * ```typescript
 * import { writeFile, BaseDirectory } from '@tauri-apps/api/fs';
 *
 * let encoder = new TextEncoder();
 * let data = encoder.encode("Hello World");
 * await writeFile('file.txt', data, { baseDir: BaseDirectory.App });
 * ```
 */
async function writeFile(
  path: string | URL,
  data: Uint8Array,
  options?: WriteFileOptions
): Promise<void> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'writeFile',
      path: path instanceof URL ? path.toString() : path,
      data: Array.from(data),
      options
    }
  })
}

/**
 * Writes UTF-8 string `data` to the given `path`, by default creating a new file if needed, else overwriting.
   @example
 * ```typescript
 * import { writeTextFile, BaseDirectory } from '@tauri-apps/api/fs';
 *
 * await writeTextFile('file.txt', "Hello world", { baseDir: BaseDirectory.App });
 * ```
 */
async function writeTextFile(
  path: string | URL,
  data: string,
  options?: WriteFileOptions
): Promise<void> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'writeTextFile',
      path: path instanceof URL ? path.toString() : path,
      data,
      options
    }
  })
}

/**
 * List directory files.
 * @example
 * ```typescript
 * import { readDir, BaseDirectory } from '@tauri-apps/api/fs';
 * // Reads the `$APPDIR/users` directory recursively
 * const entries = await readDir('users', { dir: BaseDirectory.App, recursive: true });
 *
 * function processEntries(entries) {
 *   for (const entry of entries) {
 *     console.log(`Entry: ${entry.path}`);
 *     if (entry.children) {
 *       processEntries(entry.children)
 *     }
 *   }
 * }
 * ```
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
 * and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { createDir, BaseDirectory } from '@tauri-apps/api/fs';
 * // Create the `$APPDIR/users` directory
 * await createDir('users', { dir: BaseDirectory.App, recursive: true });
 * ```
 *
 * @param dir Path to the directory to create.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
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
 * If the directory is not empty and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { removeDir, BaseDirectory } from '@tauri-apps/api/fs';
 * // Remove the directory `$APPDIR/users`
 * await removeDir('users', { dir: BaseDirectory.App });
 * ```
 *
 * @param dir Path to the directory to remove.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
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

export interface CopyFileOptions {
  /** Base directory for `fromPath`. */
  fromPathBaseDir?: BaseDirectory
  /** Base directory for `toPath`. */
  toPathBaseDir?: BaseDirectory
}

/**
 * Copies the contents and permissions of one file to another specified path, by default creating a new file if needed, else overwriting.
 * @example
 * ```typescript
 * import { copyFile, BaseDirectory } from '@tauri-apps/api/fs';
 * await copyFile('app.conf', 'app.conf.bk', { dir: BaseDirectory.App });
 * ```
 */
async function copyFile(
  fromPath: string | URL,
  toPath: string | URL,
  options?: CopyFileOptions
): Promise<void> {
  if (
    (fromPath instanceof URL && fromPath.protocol !== 'file:') ||
    (toPath instanceof URL && toPath.protocol !== 'file:')
  ) {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'copyFile',
      fromPath: fromPath instanceof URL ? fromPath.toString() : fromPath,
      toPath: toPath instanceof URL ? toPath.toString() : toPath,
      options
    }
  })
}

/**
 * Removes a file.
 * @example
 * ```typescript
 * import { removeFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Remove the `$APPDIR/app.conf` file
 * await removeFile('app.conf', { dir: BaseDirectory.App });
 * ```
 *
 * @param file Path to the file to remove.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
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
      options
    }
  })
}

/**
 * Renames a file.
 * @example
 * ```typescript
 * import { renameFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // Rename the `$APPDIR/avatar.png` file
 * await renameFile('avatar.png', 'deleted.png', { dir: BaseDirectory.App });
 * ```
 *
 * @param oldPath A path of the file to rename.
 * @param newPath A path of the new file name.
 * @param options Configuration object.
 * @returns A promise indicating the success or failure of the operation.
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

export type {
  FsOptions,
  FsDirOptions,
  FsTextFileOption,
  BinaryFileContents,
  FsBinaryFileOption,
  FileEntry
}

export {
  readTextFile,
  readFile,
  writeFile,
  writeTextFile,
  readDir,
  createDir,
  removeDir,
  copyFile,
  removeFile,
  renameFile
}
