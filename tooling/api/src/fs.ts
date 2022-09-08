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
 *         "mkdir": true,
 *         "remove": true,
 *         "rename": true
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

interface CopyFileOptions {
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
 * await copyFile('app.conf', 'app.conf.bk', { baseDir: BaseDirectory.App });
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

interface MkdirOptions {
  /** Permissions to use when creating the directory (defaults to `0o777`, before the process's umask). Ignored on Windows. */
  mode?: number
  /**
   * Defaults to `false`. If set to `true`, means that any intermediate directories will also be created (as with the shell command `mkdir -p`).
   * */
  recursive?: boolean
  /** Base directory for `path` */
  baseDir: BaseDirectory
}

/**
 * Creates a new directory with the specified path.
 * @example
 * ```typescript
 * import { mkdir, BaseDirectory } from '@tauri-apps/api/fs';
 * await mkdir('users', { baseDir: BaseDirectory.App });
 * ```
 */
async function mkdir(
  path: string | URL,
  options?: MkdirOptions
): Promise<void> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'mkdir',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })
}

interface ReadDirOptions {
  /** Base directory for `path` */
  baseDir: BaseDirectory
}

/** A disk entry which is either a file, a directory or a symlink.
 *
 * This is the result of the {@link readDir | `readDir`}.
 *
 */
interface DirEntry {
  /** The name of the entry (file name with extension or directory name). */
  name: string
  /** Specifies whether this entry is a directory or not. */
  isDirectory: boolean
  /** Specifies whether this entry is a file or not. */
  isFile: boolean
  /** Specifies whether this entry is a symlink or not. */
  isSymlink: boolean
}

/**
 * Reads the directory given by path and returns an array of `DirEntry`.
 * @example
 * ```typescript
 * import { readDir, BaseDirectory } from '@tauri-apps/api/fs';
 * const dir = "users"
 * const entries = await readDir('users', { baseDir: BaseDirectory.App });
 * processEntriesRecursive(dir, entries);
 *
 * async function processEntriesRecursive(parent, entries) {
 *   for (const entry of entries) {
 *     console.log(`Entry: ${entry.name}`);
 *     if (entry.isDirectory) {
 *        const dir = parent + entry.name;
 *       processEntriesRecursive(dir, await readDir(dir, { baseDir: BaseDirectory.App }))
 *     }
 *   }
 * }
 * ```
 */
async function readDir(
  path: string | URL,
  options?: ReadDirOptions
): Promise<DirEntry[]> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'readDir',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })
}

interface ReadFileOptions {
  /** Base directory for `path` */
  baseDir?: BaseDirectory
}

/**
 * Reads and resolves to the entire contents of a file as an array of bytes. TextDecoder can be used to transform the bytes to string if required.
 * @example
 * ```typescript
 * import { readFile, BaseDirectory } from '@tauri-apps/api/fs';
 * const contents = await readFile('avatar.png', { baseDir: BaseDirectory.Resource });
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
 * const contents = await readTextFile('app.conf', { baseDir: BaseDirectory.App });
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

interface RemoveOptions {
  /** Defaults to `false`. If set to `true`, path will be removed even if it's a non-empty directory. */
  recursive?: boolean
  /** Base directory for `path` */
  baseDir: BaseDirectory
}

/**
 * Removes the named file or directory.
 * If the directory is not empty and the `recursive` option isn't set to true, the promise will be rejected.
 * @example
 * ```typescript
 * import { remove, BaseDirectory } from '@tauri-apps/api/fs';
 * await remove('users/file.txt', { baseDir: BaseDirectory.App });
 * await remove('users', { baseDir: BaseDirectory.App });
 * ```
 */
async function remove(
  path: string | URL,
  options?: RemoveOptions
): Promise<void> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'remove',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })
}

interface RenameOptions {
  /** Base directory for `oldPath`. */
  oldPathBaseDir?: BaseDirectory
  /** Base directory for `newPath`. */
  newPathBaseDir?: BaseDirectory
}

/**
 * Renames (moves) oldpath to newpath. Paths may be files or directories.
 * If newpath already exists and is not a directory, rename() replaces it.
 * OS-specific restrictions may apply when oldpath and newpath are in different directories.
 *
 * On Unix, this operation does not follow symlinks at either path.
 *
 * @example
 * ```typescript
 * import { rename, BaseDirectory } from '@tauri-apps/api/fs';
 * await rename('avatar.png', 'deleted.png', { baseDir: BaseDirectory.App });
 * ```
 */
async function rename(
  oldPath: string | URL,
  newPath: string | URL,
  options: RenameOptions
): Promise<void> {
  if (
    (oldPath instanceof URL && oldPath.protocol !== 'file:') ||
    (newPath instanceof URL && newPath.protocol !== 'file:')
  ) {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'rename',
      oldPath: oldPath instanceof URL ? oldPath.toString() : oldPath,
      newPath: newPath instanceof URL ? newPath.toString() : newPath,
      options
    }
  })
}

interface WriteFileOptions {
  /** Defaults to false. If set to true, will append to a file instead of overwriting previous contents. */
  append?: boolean
  /** Sets the option to allow creating a new file, if one doesn't already exist at the specified path (defaults to true). */
  create?: boolean
  /** File permissions. Ignored on Windows. */
  mode?: number
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

export type {
  CopyFileOptions,
  MkdirOptions,
  DirEntry,
  ReadDirOptions,
  ReadFileOptions,
  RemoveOptions,
  RenameOptions,
  WriteFileOptions
}

export {
  copyFile,
  mkdir,
  readDir,
  readFile,
  readTextFile,
  remove,
  rename,
  writeFile,
  writeTextFile
}
