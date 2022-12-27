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
 *         "exists": true
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
 * *databases* folder of the {@link path.appDataDir | $APPDATA directory}:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "fs": {
 *         "scope": ["$APPDATA/databases/*"]
 *       }
 *     }
 *   }
 * }
 * ```
 *
 * Notice the use of the `$APPDATA` variable. The value is injected at runtime, resolving to the {@link path.appDataDir | app data directory}.
 * The available variables are:
 * {@link path.appConfigDir | `$APPCONFIG`}, {@link path.appDataDir | `$APPDATA`}, {@link path.appLocalDataDir | `$APPLOCALDATA`},
 * {@link path.appCacheDir | `$APPCACHE`}, {@link path.appLogDir | `$APPLOG`},
 * {@link path.audioDir | `$AUDIO`}, {@link path.cacheDir | `$CACHE`}, {@link path.configDir | `$CONFIG`}, {@link path.dataDir | `$DATA`},
 * {@link path.localDataDir | `$LOCALDATA`}, {@link path.desktopDir | `$DESKTOP`}, {@link path.documentDir | `$DOCUMENT`},
 * {@link path.downloadDir | `$DOWNLOAD`}, {@link path.executableDir | `$EXE`}, {@link path.fontDir | `$FONT`}, {@link path.homeDir | `$HOME`},
 * {@link path.pictureDir | `$PICTURE`}, {@link path.publicDir | `$PUBLIC`}, {@link path.runtimeDir | `$RUNTIME`},
 * {@link path.templateDir | `$TEMPLATE`}, {@link path.videoDir | `$VIDEO`}, {@link path.resourceDir | `$RESOURCE`}, {@link path.appDir | `$APP`},
 * {@link path.logDir | `$LOG`}, {@link path.tempDir | `$TEMP`}.
 *
 * Trying to execute any API with a URL not configured on the scope results in a promise rejection due to denied access.
 *
 * Note that this scope applies to **all** APIs on this module.
 *
 * @module
 */

import { isWindows } from './helpers/os-check'
import { invokeTauriCommand } from './helpers/tauri'

enum BaseDirectory {
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
  Temp,
  AppConfig,
  AppData,
  AppLocalData,
  AppCache,
  AppLog
}

enum SeekMode {
  Start = 0,
  Current = 1,
  End = 2
}

/**
 * A FileInfo describes a file and is returned by `stat`, `lstat` or `fstat`.
 */
interface FileInfo {
  /**
   * True if this is info for a regular file. Mutually exclusive to
   * `FileInfo.isDirectory` and `FileInfo.isSymlink`.
   */
  isFile: boolean
  /**
   * True if this is info for a regular directory. Mutually exclusive to
   * `FileInfo.isFile` and `FileInfo.isSymlink`.
   */
  isDirectory: boolean
  /**
   * True if this is info for a symlink. Mutually exclusive to
   * `FileInfo.isFile` and `FileInfo.isDirectory`.
   */
  isSymlink: boolean
  /**
   * The size of the file, in bytes.
   */
  size: number
  /**
   * The last modification time of the file. This corresponds to the `mtime`
   * field from `stat` on Linux/Mac OS and `ftLastWriteTime` on Windows. This
   * may not be available on all platforms.
   */
  mtime: Date | null
  /**
   * The last access time of the file. This corresponds to the `atime`
   * field from `stat` on Unix and `ftLastAccessTime` on Windows. This may not
   * be available on all platforms.
   */
  atime: Date | null
  /**
   * The creation time of the file. This corresponds to the `birthtime`
   * field from `stat` on Mac/BSD and `ftCreationTime` on Windows. This may
   * not be available on all platforms.
   */
  birthtime: Date | null
  /**
   * ID of the device containing the file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  dev: number | null
  /**
   * Inode number.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  ino: number | null
  /**
   * The underlying raw `st_mode` bits that contain the standard Unix
   * permissions for this file/directory.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  mode: number | null
  /**
   * Number of hard links pointing to this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  nlink: number | null
  /**
   * User ID of the owner of this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  uid: number | null
  /**
   * Group ID of the owner of this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  gid: number | null
  /**
   * Device ID of this file.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  rdev: number | null
  /**
   * Blocksize for filesystem I/O.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  blksize: number | null
  /**
   * Number of blocks allocated to the file, in 512-byte units.
   *
   * #### Platform-specific
   *
   * - **Windows:** Unsupported.
   */
  blocks: number | null
}

function parseFileInfo(response: {
  isFile: boolean
  isDirectory: boolean
  isSymlink: boolean
  size: number
  mtime: Date | null
  atime: Date | null
  birthtime: Date | null
  dev: number
  ino: number
  mode: number
  nlink: number
  uid: number
  gid: number
  rdev: number
  blksize: number
  blocks: number
}): FileInfo {
  const unix = !isWindows()
  return {
    isFile: response.isFile,
    isDirectory: response.isDirectory,
    isSymlink: response.isSymlink,
    size: response.size,
    mtime: response.mtime != null ? new Date(response.mtime) : null,
    atime: response.atime != null ? new Date(response.atime) : null,
    birthtime: response.birthtime != null ? new Date(response.birthtime) : null,
    // Only non-null if on Unix
    dev: unix ? response.dev : null,
    ino: unix ? response.ino : null,
    mode: unix ? response.mode : null,
    nlink: unix ? response.nlink : null,
    uid: unix ? response.uid : null,
    gid: unix ? response.gid : null,
    rdev: unix ? response.rdev : null,
    blksize: unix ? response.blksize : null,
    blocks: unix ? response.blocks : null
  }
}

/** The Tauri abstraction for reading and writing files. */
class FsFile {
  readonly rid: number

  constructor(rid: number) {
    this.rid = rid
  }

  /**
   * Reads up to `p.byteLength` bytes into `p`. It resolves to the number of
   * bytes read (`0` < `n` <= `p.byteLength`) and rejects if any error
   * encountered. Even if `read()` resolves to `n` < `p.byteLength`, it may
   * use all of `p` as scratch space during the call. If some data is
   * available but not `p.byteLength` bytes, `read()` conventionally resolves
   * to what is available instead of waiting for more.
   *
   * When `read()` encounters end-of-file condition, it resolves to EOF
   * (`null`).
   *
   * When `read()` encounters an error, it rejects with an error.
   *
   * Callers should always process the `n` > `0` bytes returned before
   * considering the EOF (`null`). Doing so correctly handles I/O errors that
   * happen after reading some bytes and also both of the allowed EOF
   * behaviors.
   */
  async read(p: Uint8Array): Promise<number | null> {
    return read(this.rid, p)
  }

  /**
   * Seek sets the offset for the next `read()` or `write()` to offset,
   * interpreted according to `whence`: `Start` means relative to the
   * start of the file, `Current` means relative to the current offset,
   * and `End` means relative to the end. Seek resolves to the new offset
   * relative to the start of the file.
   *
   * Seeking to an offset before the start of the file is an error. Seeking to
   * any positive offset is legal, but the behavior of subsequent I/O
   * operations on the underlying object is implementation-dependent.
   * It returns the number of cursor position.
   */
  async seek(offset: number, whence: SeekMode): Promise<number> {
    return seek(this.rid, offset, whence)
  }

  /**
   * Returns a {@link FileInfo |`FileInfo`} for this file.
   */
  async stat(): Promise<FileInfo> {
    return fstat(this.rid)
  }

  /**
   * Truncates or extends this file, to reach the specified `len`.
   * If `len` is not specified then the entire file contents are truncated.
   */
  async truncate(len?: number): Promise<void> {
    return ftruncate(this.rid, len)
  }

  /**
   * Writes `p.byteLength` bytes from `p` to the underlying data stream. It
   * resolves to the number of bytes written from `p` (`0` <= `n` <=
   * `p.byteLength`) or reject with the error encountered that caused the
   * write to stop early. `write()` must reject with a non-null error if
   * would resolve to `n` < `p.byteLength`. `write()` must not modify the
   * slice data, even temporarily.
   */
  async write(p: Uint8Array): Promise<number> {
    return write(this.rid, p)
  }

  async close(): Promise<void> {
    return close(this.rid)
  }
}

interface CreateOptions {
  /** Base directory for `path` */
  baseDir?: BaseDirectory
}

/**
 * Creates a file if none exists or truncates an existing file and resolves to
 *  an instance of {@link FsFile | `FsFile` }.
 *
 * @example
 * ```typescript
 * import { create, BaseDirectory } from "@tauri-apps/api/fs"
 * const file = await create("foo/bar.txt", { baseDir: BaseDirectory.App });
 * ```
 */
async function create(
  path: string | URL,
  options?: CreateOptions
): Promise<FsFile> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  const rid = await invokeTauriCommand<number>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'create',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })

  return new FsFile(rid)
}

interface OpenOptions {
  /**
   * Sets the option for read access. This option, when `true`, means that the
   * file should be read-able if opened.
   */
  read?: boolean
  /**
   * Sets the option for write access. This option, when `true`, means that
   * the file should be write-able if opened. If the file already exists,
   * any write calls on it will overwrite its contents, by default without
   * truncating it.
   */
  write?: boolean
  /**
   * Sets the option for the append mode. This option, when `true`, means that
   * writes will append to a file instead of overwriting previous contents.
   * Note that setting `{ write: true, append: true }` has the same effect as
   * setting only `{ append: true }`.
   */
  append?: boolean
  /**
   * Sets the option for truncating a previous file. If a file is
   * successfully opened with this option set it will truncate the file to `0`
   * size if it already exists. The file must be opened with write access
   * for truncate to work.
   */
  truncate?: boolean
  /**
   * Sets the option to allow creating a new file, if one doesn't already
   * exist at the specified path. Requires write or append access to be
   * used.
   */
  create?: boolean
  /**
   * Defaults to `false`. If set to `true`, no file, directory, or symlink is
   * allowed to exist at the target location. Requires write or append
   * access to be used. When createNew is set to `true`, create and truncate
   * are ignored.
   */
  createNew?: boolean
  /**
   * Permissions to use if creating the file (defaults to `0o666`, before
   * the process's umask).
   * Ignored on Windows.
   */
  mode?: number
  /** Base directory for `path` */
  baseDir?: BaseDirectory
}

/**
 * Open a file and resolve to an instance of {@link FsFile | `FsFile`}.  The
 * file does not need to previously exist if using the `create` or `createNew`
 * open options. It is the callers responsibility to close the file when finished
 * with it.
 *
 * @example
 * ```typescript
 * import { open, BaseDirectory } from "@tauri-apps/api/fs"
 * const file = await open("foo/bar.txt", { read: true, write: true, baseDir: BaseDirectory.App });
 * // Do work with file
 * await close(file.rid);
 * ```
 */
async function open(
  path: string | URL,
  options?: OpenOptions
): Promise<FsFile> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  const rid = await invokeTauriCommand<number>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'open',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })

  return new FsFile(rid)
}

/**
 * Close the given resource ID (rid) which has been previously opened, such
 * as via opening or creating a file. Closing a file when you are finished
 * with it is important to avoid leaking resources.
 *
 * @example
 * ```typescript
 * import { open, close, BaseDirectory } from "@tauri-apps/api/fs"
 * const file = await open("my_file.txt", { baseDir: BaseDirectory.App });
 * // do work with "file" object
 * await close(file.rid);
 * ```
 */
async function close(rid: number): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'close',
      rid
    }
  })
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

/**
 * A disk entry which is either a file, a directory or a symlink.
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

/**
 *  Read from a resource ID (`rid`) into an array buffer (`buffer`).
 *
 * Resolves to either the number of bytes read during the operation or EOF
 * (`null`) if there was nothing more to read.
 *
 * It is possible for a read to successfully return with `0` bytes. This does
 * not indicate EOF.
 *
 *
 * **It is not guaranteed that the full buffer will be read in a single call.**
 *
 * @example
 * ```typescript
 * import { open, read, close, BaseDirectory } from "@tauri-apps/api/fs"
 * // if "$APP/foo/bar.txt" contains the text "hello world":
 * const file = await open("foo/bar.txt", { baseDir: BaseDirectory.App });
 * const buf = new Uint8Array(100);
 * const numberOfBytesRead = await read(file.rid, buf); // 11 bytes
 * const text = new TextDecoder().decode(buf);  // "hello world"
 * await close(file.rid);
 * ```
 */
async function read(rid: number, buffer: Uint8Array): Promise<number | null> {
  if (buffer.byteLength === 0) {
    return 0
  }

  const [data, nread] = await invokeTauriCommand<[number[], number]>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'read',
      rid,
      len: buffer.byteLength
    }
  })

  buffer.set(data)

  return nread === 0 ? null : nread
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

/**
 * Seek a resource ID (`rid`) to the given `offset` under mode given by `whence`.
 * The call resolves to the new position within the resource (bytes from the start).
 *
 * @example
 * ```typescript
 * import { open, seek, write, SeekMode, BaseDirectory } from '@tauri-apps/api/fs';
 *
 * // Given file.rid pointing to file with "Hello world", which is 11 bytes long:
 * const file = await open('hello.txt', { read: true, write: true, truncate: true, create: true, baseDir: BaseDirectory.App });
 * await write(file.rid, new TextEncoder().encode("Hello world"));
 *
 * // advance cursor 6 bytes
 * const cursorPosition = await seek(file.rid, 6, SeekMode.Start);
 * console.log(cursorPosition);  // 6
 * const buf = new Uint8Array(100);
 * await file.read(buf);
 * console.log(new TextDecoder().decode(buf)); // "world"
 * ```
 *
 * The seek modes work as follows:
 *
 * ```typescript
 * import { open, seek, write, SeekMode, BaseDirectory } from '@tauri-apps/api/fs';
 *
 * // Given file.rid pointing to file with "Hello world", which is 11 bytes long:
 * const file = await open('hello.txt', { read: true, write: true, truncate: true, create: true, baseDir: BaseDirectory.App });
 * await write(file.rid, new TextEncoder().encode("Hello world"), { baseDir: BaseDirectory.App });
 *
 * // Seek 6 bytes from the start of the file
 * console.log(await seek(file.rid, 6, SeekMode.Start)); // "6"
 * // Seek 2 more bytes from the current position
 * console.log(await seek(file.rid, 2, SeekMode.Current)); // "8"
 * // Seek backwards 2 bytes from the end of the file
 * console.log(await seek(file.rid, -2, SeekMode.End)); // "9" (e.g. 11-2)
 * ```
 */
async function seek(
  rid: number,
  offset: number,
  whence: SeekMode
): Promise<number> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'seek',
      rid,
      offset,
      whence
    }
  })
}

interface StatOptions {
  /** Base directory for `path`. */
  baseDir: BaseDirectory
}

/**
 * Resolves to a {@link FileInfo | `FileInfo`} for the specified `path`. Will always
 * follow symlinks but will reject if the symlink points to a path outside of the scope.
 *
 * @example
 * ```typescript
 * import { stat, BaseDirectory } from '@tauri-apps/api/fs';
 * const fileInfo = await stat("hello.txt", { baseDir: BaseDirectory.App });
 * console.log(fileInfo.isFile); // true
 * ```
 */
async function stat(
  path: string | URL,
  options?: StatOptions
): Promise<FileInfo> {
  const res = await invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'stat',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })

  return parseFileInfo(res as any)
}

/**
 * Resolves to a {@link FileInfo | `FileInfo`} for the specified `path`. If `path` is a
 * symlink, information for the symlink will be returned instead of what it
 * points to.
 *
 * @example
 * ```typescript
 * import { lstat, BaseDirectory } from '@tauri-apps/api/fs';
 * const fileInfo = await lstat("hello.txt", { baseDir: BaseDirectory.App });
 * console.log(fileInfo.isFile); // true
 * ```
 */
async function lstat(
  path: string | URL,
  options?: StatOptions
): Promise<FileInfo> {
  const res = await invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'lstat',
      path: path instanceof URL ? path.toString() : path,
      options
    }
  })

  return parseFileInfo(res as any)
}

/**
 * Returns a {@link FileInfo | `FileInfo`} for the given file stream.
 *
 * @example
 * ```typescript
 * import { open, fstat, BaseDirectory } from '@tauri-apps/api/fs';
 * const file = await open("file.txt", { read: true, baseDir: BaseDirectory.App });
 * const fileInfo = await fstat(file.rid);
 * console.log(fileInfo.isFile); // true
 * ```
 */
async function fstat(rid: number): Promise<FileInfo> {
  const res = await invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'fstat',
      rid
    }
  })

  return parseFileInfo(res as any)
}

interface TruncateOptions {
  /** Base directory for `path`. */
  baseDir: BaseDirectory
}

/**
 * Truncates or extends the specified file, to reach the specified `len`.
 * If `len` is `0` or not specified, then the entire file contents are truncated.
 *
 * @example
 * ```typescript
 * import { truncate, readFile, writeFile, BaseDirectory } from '@tauri-apps/api/fs';
 * // truncate the entire file
 * await truncate("my_file.txt", 0, { baseDir: BaseDirectory.App });
 *
 * // truncate part of the file
 * let file = "file.txt";
 * await writeFile(file, new TextEncoder().encode("Hello World"), { baseDir: BaseDirectory.App });
 * await truncate(file, 7);
 * const data = await readFile(file, { baseDir: BaseDirectory.App });
 * console.log(new TextDecoder().decode(data));  // "Hello W"
 * ```
 */
async function truncate(
  path: string | URL,
  len?: number,
  options?: TruncateOptions
): Promise<void> {
  if (path instanceof URL && path.protocol !== 'file:') {
    throw new TypeError('Must be a file URL.')
  }

  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'truncate',
      path: path instanceof URL ? path.toString() : path,
      len,
      options
    }
  })
}

/**
 * Truncates or extends the specified file stream, to reach the specified `len`.
 *
 * If `len` is `0` or not specified then the entire file contents are truncated as if len was set to 0.
 *
 * If the file previously was larger than this new length, the extra  data  is  lost.
 *
 * If  the  file  previously  was shorter, it is extended, and the extended part reads as null bytes ('\0').
 *
 * @example
 * ```typescript
 * import { ftruncate, open, write, read, BaseDirectory } from '@tauri-apps/api/fs';
 *
 * // truncate the entire file
 * const file = await open("my_file.txt", { read: true, write: true, create: true, baseDir: BaseDirectory.App });
 * await ftruncate(file.rid);
 *
 * // truncate part of the file
 * const file = await open("my_file.txt", { read: true, write: true, create: true, baseDir: BaseDirectory.App });
 * await write(file.rid, new TextEncoder().encode("Hello World"));
 * await ftruncate(file.rid, 7);
 * const data = new Uint8Array(32);
 * await read(file.rid, data);
 * console.log(new TextDecoder().decode(data)); // Hello W
 * ```
 */
async function ftruncate(rid: number, len?: number): Promise<void> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'ftruncate',
      rid,
      len
    }
  })
}

/**
 * Write to the resource ID (`rid`) the contents of the array buffer (`data`).
 *
 * Resolves to the number of bytes written.
 *
 * **It is not guaranteed that the full buffer will be written in a single
 * call.**
 *
 * @example
 * ```typescript
 * import { open, write, close, BaseDirectory } from '@tauri-apps/api/fs';
 * const encoder = new TextEncoder();
 * const data = encoder.encode("Hello world");
 * const file = await open("bar.txt", { write: true, baseDir: BaseDirectory.App });
 * const bytesWritten = await write(file.rid, data); // 11
 * await close(file.rid);
 * ```
 */
async function write(rid: number, data: Uint8Array): Promise<number> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'write',
      rid,
      data: Array.from(data)
    }
  })
}

interface WriteFileOptions {
  /** Defaults to `false`. If set to `true`, will append to a file instead of overwriting previous contents. */
  append?: boolean
  /** Sets the option to allow creating a new file, if one doesn't already exist at the specified path (defaults to `true`). */
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

interface ExistsOptions {
  /** Base directory for `path`. */
  baseDir: BaseDirectory
}

/**
 * Check if a path exists.
 * @example
 * ```typescript
 * import { exists, BaseDirectory } from '@tauri-apps/api/fs';
 * // Check if the `$APPDATA/avatar.png` file exists
 * await exists('avatar.png', { dir: BaseDirectory.AppData });
 * ```
 *
 * @since 1.1.0
 */
async function exists(path: string, options?: ExistsOptions): Promise<boolean> {
  return invokeTauriCommand({
    __tauriModule: 'Fs',
    message: {
      cmd: 'exists',
      path,
      options
    }
  })
}

export type {
  CreateOptions,
  OpenOptions,
  CopyFileOptions,
  MkdirOptions,
  DirEntry,
  ReadDirOptions,
  ReadFileOptions,
  RemoveOptions,
  RenameOptions,
  StatOptions,
  TruncateOptions,
  WriteFileOptions,
  ExistsOptions,
  FileInfo
}

export {
  BaseDirectory,
  FsFile,
  create,
  open,
  close,
  copyFile,
  mkdir,
  read,
  readDir,
  readFile,
  readTextFile,
  remove,
  rename,
  SeekMode,
  seek,
  stat,
  lstat,
  fstat,
  truncate,
  ftruncate,
  write,
  writeFile,
  writeTextFile,
  exists
}
