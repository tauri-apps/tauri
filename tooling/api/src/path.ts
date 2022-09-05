// Copyright 2019-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The path module provides utilities for working with file and directory paths.
 *
 * This package is also accessible with `window.__TAURI__.path` when [`build.withGlobalTauri`](https://tauri.app/v1/api/config/#buildconfig.withglobaltauri) in `tauri.conf.json` is set to `true`.
 *
 * The APIs must be added to [`tauri.allowlist.path`](https://tauri.app/v1/api/config/#allowlistconfig.path) in `tauri.conf.json`:
 * ```json
 * {
 *   "tauri": {
 *     "allowlist": {
 *       "path": {
 *         "all": true, // enable all Path APIs
 *       }
 *     }
 *   }
 * }
 * ```
 * It is recommended to allowlist only the APIs you use for optimal bundle size and security.
 * @module
 */

import { invokeTauriCommand } from './helpers/tauri'
import { BaseDirectory } from './fs'
import { isWindows } from './helpers/os-check'

/**
 * Returns the path to the suggested directory for your app config files.
 * Resolves to `${configDir}/${bundleIdentifier}`, where `bundleIdentifier` is the value [`tauri.bundle.identifier`](https://tauri.app/v1/api/config/#bundleconfig.identifier) is configured in `tauri.conf.json`.
 * @example
 * ```typescript
 * import { appDir } from '@tauri-apps/api/path';
 * const appDirPath = await appDir();
 * ```
 *
 * @returns
 */
async function appDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.App
    }
  })
}

/**
 * Returns the path to the user's audio directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_MUSIC_DIR`.
 * - **macOS:** Resolves to `$HOME/Music`.
 * - **Windows:** Resolves to `{FOLDERID_Music}`.
 * @example
 * ```typescript
 * import { audioDir } from '@tauri-apps/api/path';
 * const audioDirPath = await audioDir();
 * ```
 *
 * @returns
 */
async function audioDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Audio
    }
  })
}

/**
 * Returns the path to the user's cache directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_CACHE_HOME` or `$HOME/.cache`.
 * - **macOS:** Resolves to `$HOME/Library/Caches`.
 * - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
 * @example
 * ```typescript
 * import { cacheDir } from '@tauri-apps/api/path';
 * const cacheDirPath = await cacheDir();
 * ```
 *
 * @returns
 */
async function cacheDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Cache
    }
  })
}

/**
 * Returns the path to the user's config directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`.
 * - **macOS:** Resolves to `$HOME/Library/Application Support`.
 * - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
 * @example
 * ```typescript
 * import { configDir } from '@tauri-apps/api/path';
 * const configDirPath = await configDir();
 * ```
 *
 * @returns
 */
async function configDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Config
    }
  })
}

/**
 * Returns the path to the user's data directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
 * - **macOS:** Resolves to `$HOME/Library/Application Support`.
 * - **Windows:** Resolves to `{FOLDERID_RoamingAppData}`.
 * @example
 * ```typescript
 * import { dataDir } from '@tauri-apps/api/path';
 * const dataDirPath = await dataDir();
 * ```
 *
 * @returns
 */
async function dataDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Data
    }
  })
}

/**
 * Returns the path to the user's desktop directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DESKTOP_DIR`.
 * - **macOS:** Resolves to `$HOME/Library/Desktop`.
 * - **Windows:** Resolves to `{FOLDERID_Desktop}`.
 * @example
 * ```typescript
 * import { desktopDir } from '@tauri-apps/api/path';
 * const desktopPath = await desktopDir();
 * ```

 * @returns
 */
async function desktopDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Desktop
    }
  })
}

/**
 * Returns the path to the user's document directory.
 * @example
 * ```typescript
 * import { documentDir } from '@tauri-apps/api/path';
 * const documentDirPath = await documentDir();
 * ```
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOCUMENTS_DIR`.
 * - **macOS:** Resolves to `$HOME/Documents`.
 * - **Windows:** Resolves to `{FOLDERID_Documents}`.
 *
 * @returns
 */
async function documentDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Document
    }
  })
}

/**
 * Returns the path to the user's download directory.
 *
 * #### Platform-specific
 *
 * - **Linux**: Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_DOWNLOAD_DIR`.
 * - **macOS**: Resolves to `$HOME/Downloads`.
 * - **Windows**: Resolves to `{FOLDERID_Downloads}`.
 * @example
 * ```typescript
 * import { downloadDir } from '@tauri-apps/api/path';
 * const downloadDirPath = await downloadDir();
 * ```
 *
 * @returns
 */
async function downloadDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Download
    }
  })
}

/**
 * Returns the path to the user's executable directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_BIN_HOME/../bin` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`.
 * - **macOS:** Not supported.
 * - **Windows:** Not supported.
 * @example
 * ```typescript
 * import { executableDir } from '@tauri-apps/api/path';
 * const executableDirPath = await executableDir();
 * ```
 *
 * @returns
 */
async function executableDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Executable
    }
  })
}

/**
 * Returns the path to the user's font directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_DATA_HOME/fonts` or `$HOME/.local/share/fonts`.
 * - **macOS:** Resolves to `$HOME/Library/Fonts`.
 * - **Windows:** Not supported.
 * @example
 * ```typescript
 * import { fontDir } from '@tauri-apps/api/path';
 * const fontDirPath = await fontDir();
 * ```
 *
 * @returns
 */
async function fontDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Font
    }
  })
}

/**
 * Returns the path to the user's home directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$HOME`.
 * - **macOS:** Resolves to `$HOME`.
 * - **Windows:** Resolves to `{FOLDERID_Profile}`.
 * @example
 * ```typescript
 * import { homeDir } from '@tauri-apps/api/path';
 * const homeDirPath = await homeDir();
 * ```
 *
 * @returns
 */
async function homeDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Home
    }
  })
}

/**
 * Returns the path to the user's local data directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
 * - **macOS:** Resolves to `$HOME/Library/Application Support`.
 * - **Windows:** Resolves to `{FOLDERID_LocalAppData}`.
 * @example
 * ```typescript
 * import { localDataDir } from '@tauri-apps/api/path';
 * const localDataDirPath = await localDataDir();
 * ```
 *
 * @returns
 */
async function localDataDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.LocalData
    }
  })
}

/**
 * Returns the path to the user's picture directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PICTURES_DIR`.
 * - **macOS:** Resolves to `$HOME/Pictures`.
 * - **Windows:** Resolves to `{FOLDERID_Pictures}`.
 * @example
 * ```typescript
 * import { pictureDir } from '@tauri-apps/api/path';
 * const pictureDirPath = await pictureDir();
 * ```
 *
 * @returns
 */
async function pictureDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Picture
    }
  })
}

/**
 * Returns the path to the user's public directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_PUBLICSHARE_DIR`.
 * - **macOS:** Resolves to `$HOME/Public`.
 * - **Windows:** Resolves to `{FOLDERID_Public}`.
 * @example
 * ```typescript
 * import { publicDir } from '@tauri-apps/api/path';
 * const publicDirPath = await publicDir();
 * ```
 *
 * @returns
 */
async function publicDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Public
    }
  })
}

/**
 * Returns the path to the application's resource directory.
 * To resolve a resource path, see the [[resolveResource | `resolveResource API`]].
 * @example
 * ```typescript
 * import { resourceDir } from '@tauri-apps/api/path';
 * const resourceDirPath = await resourceDir();
 * ```
 *
 * @returns
 */
async function resourceDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Resource
    }
  })
}

/**
 * Resolve the path to a resource file.
 * @example
 * ```typescript
 * import { resolveResource } from '@tauri-apps/api/path';
 * const resourcePath = await resolveResource('script.sh');
 * ```
 *
 * @param resourcePath The path to the resource.
 * Must follow the same syntax as defined in `tauri.conf.json > tauri > bundle > resources`, i.e. keeping subfolders and parent dir components (`../`).
 * @returns The full path to the resource.
 */
async function resolveResource(resourcePath: string): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: resourcePath,
      directory: BaseDirectory.Resource
    }
  })
}

/**
 * Returns the path to the user's runtime directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `$XDG_RUNTIME_DIR`.
 * - **macOS:** Not supported.
 * - **Windows:** Not supported.
 * @example
 * ```typescript
 * import { runtimeDir } from '@tauri-apps/api/path';
 * const runtimeDirPath = await runtimeDir();
 * ```
 *
 * @returns
 */
async function runtimeDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Runtime
    }
  })
}

/**
 * Returns the path to the user's template directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_TEMPLATES_DIR`.
 * - **macOS:** Not supported.
 * - **Windows:** Resolves to `{FOLDERID_Templates}`.
 * @example
 * ```typescript
 * import { templateDir } from '@tauri-apps/api/path';
 * const templateDirPath = await templateDir();
 * ```
 *
 * @returns
 */
async function templateDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Template
    }
  })
}

/**
 * Returns the path to the user's video directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to [`xdg-user-dirs`](https://www.freedesktop.org/wiki/Software/xdg-user-dirs/)' `XDG_VIDEOS_DIR`.
 * - **macOS:** Resolves to `$HOME/Movies`.
 * - **Windows:** Resolves to `{FOLDERID_Videos}`.
 * @example
 * ```typescript
 * import { videoDir } from '@tauri-apps/api/path';
 * const videoDirPath = await videoDir();
 * ```
 *
 * @returns
 */
async function videoDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Video
    }
  })
}

/**
 * Returns the path to the suggested log directory.
 *
 * #### Platform-specific
 *
 * - **Linux:** Resolves to `${configDir}/${bundleIdentifier}`.
 * - **macOS:** Resolves to `${homeDir}//Library/Logs/{bundleIdentifier}`
 * - **Windows:** Resolves to `${configDir}/${bundleIdentifier}`.
 * @example
 * ```typescript
 * import { logDir } from '@tauri-apps/api/path';
 * const logDirPath = await logDir();
 * ```
 *
 * @returns
 */
async function logDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Log
    }
  })
}

/**
 * Returns the operating system's default directory for temporary files as a string.
 * @example
 * ```typescript
 * import { tempdir } from '@tauri-apps/api/path';
 * const tempdirPath = await tempdir();
 * ```
 */
async function tempDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'tempdir'
    }
  })
}

/**
 * Provides the platform-specific path segment separator:
 * - `\` on Windows
 * - `/` on POSIX
 */
const sep = isWindows() ? '\\' : '/'

/**
 * Provides the platform-specific path segment delimiter:
 * - `;` on Windows
 * - `:` on POSIX
 */
const delimiter = isWindows() ? ';' : ':'

/**
 * Resolves a sequence of `paths` or `path` segments into an absolute path.
 * @example
 * ```typescript
 * import { resolve, appDir } from '@tauri-apps/api/path';
 * const appDirPath = await appDir();
 * const path = await resolve(appDirPath, '..', 'users', 'tauri', 'avatar.png');
 * ```
 *
 * @param paths A sequence of paths or path segments.
 */
async function resolve(...paths: string[]): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'resolve',
      paths
    }
  })
}

/**
 * Normalizes the given `path`, resolving `'..'` and `'.'` segments and resolve symbolic links.
 * @example
 * ```typescript
 * import { normalize, appDir } from '@tauri-apps/api/path';
 * const appDirPath = await appDir();
 * const path = await normalize(appDirPath, '..', 'users', 'tauri', 'avatar.png');
 * ```
 */
async function normalize(path: string): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'normalize',
      path
    }
  })
}

/**
 *  Joins all given `path` segments together using the platform-specific separator as a delimiter, then normalizes the resulting path.
 * @example
 * ```typescript
 * import { join, appDir } from '@tauri-apps/api/path';
 * const appDirPath = await appDir();
 * const path = await join(appDirPath, 'users', 'tauri', 'avatar.png');
 * ```
 *
 * @param paths A sequence of path segments.
 */
async function join(...paths: string[]): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'join',
      paths
    }
  })
}

/**
 * Returns the directory name of a `path`. Trailing directory separators are ignored.
 * @example
 * ```typescript
 * import { dirname, appDir } from '@tauri-apps/api/path';
 * const appDirPath = await appDir();
 * const dir = await dirname(appDirPath);
 * ```
 */
async function dirname(path: string): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'dirname',
      path
    }
  })
}

/**
 * Returns the extension of the `path`.
 * @example
 * ```typescript
 * import { extname, resolveResource } from '@tauri-apps/api/path';
 * const resourcePath = await resolveResource('app.conf');
 * const ext = await extname(resourcePath);
 * assert(ext === 'conf');
 * ```
 */
async function extname(path: string): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'extname',
      path
    }
  })
}

/**
 * Returns the last portion of a `path`. Trailing directory separators are ignored.
 * @example
 * ```typescript
 * import { basename, resolveResource } from '@tauri-apps/api/path';
 * const resourcePath = await resolveResource('app.conf');
 * const base = await basename(resourcePath);
 * assert(base === 'app');
 * ```
 *
 * @param ext An optional file extension to be removed from the returned path.
 */
async function basename(path: string, ext?: string): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Path',
    message: {
      cmd: 'basename',
      path,
      ext
    }
  })
}

/**
 * Returns whether the path is absolute or not.
 * @example
 * ```typescript
 * import { isAbsolute } from '@tauri-apps/api/path';
 * assert(await isAbsolute('/home/tauri'));
 * ```
 */
async function isAbsolute(path: string): Promise<boolean> {
  return invokeTauriCommand<boolean>({
    __tauriModule: 'Path',
    message: {
      cmd: 'isAbsolute',
      path
    }
  })
}

export {
  appDir,
  audioDir,
  cacheDir,
  configDir,
  dataDir,
  desktopDir,
  documentDir,
  downloadDir,
  executableDir,
  fontDir,
  homeDir,
  localDataDir,
  pictureDir,
  publicDir,
  resourceDir,
  resolveResource,
  runtimeDir,
  templateDir,
  videoDir,
  logDir,
  tempDir,
  BaseDirectory,
  sep,
  delimiter,
  resolve,
  normalize,
  join,
  dirname,
  extname,
  basename,
  isAbsolute
}
