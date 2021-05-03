// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { BaseDirectory } from './fs'

/**
 * Read common system paths such as home, config and cache directories.
 * @packageDocumentation
 */

/**
 * Returns the path to the suggested directory for your app config files.
 * Resolves to `${configDir}/${bundleIdentifier}`, where `bundleIdentifier` is the value configured on `tauri.conf.json > tauri > bundle > identifier`.
 *
 * @returns
 */
async function appDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.App
    }
  })
}

/**
 * Returns the path to the user's audio directory.
 * # Linux
 * Resolves to `$XDG_MUSIC_DIR`.
 * # macOS
 * Resolves to `$HOME/Music`.
 * # Windows
 * Resolves to `{FOLDERID_Music}`.
 *
 * @returns
 */
async function audioDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Audio
    }
  })
}

/**
 * Returns the path to the user's cache directory.
 * # Linux
 * Resolves to `$XDG_CACHE_HOME` or `$HOME/.cache`.
 * # macOS
 * Resolves to `$HOME/Library/Caches`.
 * # Windows
 * Resolves to `{FOLDERID_LocalAppData}`.
 *
 * @returns
 */
async function cacheDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Cache
    }
  })
}

/**
 * Returns the path to the user's config directory.
 * # Linux
 * Resolves to `$XDG_CONFIG_HOME` or `$HOME/.config`.
 * # macOS
 * Resolves to `$HOME/Library/Application Support`.
 * # Windows
 * Resolves to `{FOLDERID_LocalAppData}`.
 *
 * @returns
 */
async function configDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Config
    }
  })
}

/**
 * Returns the path to the user's data directory.
 * # Linux
 * Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
 * # macOS
 * Resolves to `$HOME/Library/Application Support`.
 * # Windows
 * Resolves to `{FOLDERID_RoamingAppData}`.
 *
 * @returns
 */
async function dataDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Data
    }
  })
}

/**
 * Returns the path to the user's desktop directory.
 * # Linux
 * Resolves to `$XDG_DESKTOP_DIR`.
 * # macOS
 * Resolves to `$HOME/Library/Desktop`.
 * # Windows
 * Resolves to `{FOLDERID_Desktop}`.

 * @returns
 */
async function desktopDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Desktop
    }
  })
}

/**
 * Returns the path to the user's document directory.
 * # Linux
 * Resolves to `$XDG_DOCUMENTS_DIR`.
 * # macOS
 * Resolves to `$HOME/Documents`.
 * # Windows
 * Resolves to `{FOLDERID_Documents}`.
 *
 * @returns
 */
async function documentDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Document
    }
  })
}

/**
 * Returns the path to the user's download directory.
 * # Linux
 * Resolves to `$XDG_DOWNLOAD_DIR`.
 * # macOS
 * Resolves to `$HOME/Downloads`.
 * # Windows
 * Resolves to `{FOLDERID_Downloads}`.
 *
 * @returns
 */
async function downloadDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Download
    }
  })
}

/**
 * Returns the path to the user's executable directory.
 * # Linux
 * Resolves to `$XDG_BIN_HOME/../bin` or `$XDG_DATA_HOME/../bin` or `$HOME/.local/bin`.
 * # macOS
 * Not supported.
 * # Windows
 * Not supported.
 *
 * @returns
 */
async function executableDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Executable
    }
  })
}

/**
 * Returns the path to the user's font directory.
 * # Linux
 * Resolves to `$XDG_DATA_HOME/fonts` or `$HOME/.local/share/fonts`.
 * # macOS
 * Resolves to `$HOME/Library/Fonts`.
 * # Windows
 * Not supported.
 *
 * @returns
 */
async function fontDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Font
    }
  })
}

/**
 * Returns the path to the user's home directory.
 * # Linux
 * Resolves to `$HOME`.
 * # macOS
 * Resolves to `$HOME`.
 * # Windows
 * Resolves to `{FOLDERID_Profile}`.
 *
 * @returns
 */
async function homeDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Home
    }
  })
}

/**
 * Returns the path to the user's local data directory.
 * # Linux
 * Resolves to `$XDG_DATA_HOME` or `$HOME/.local/share`.
 * # macOS
 * Resolves to `$HOME/Library/Application Support`.
 * # Windows
 * Resolves to `{FOLDERID_LocalAppData}`.
 *
 * @returns
 */
async function localDataDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.LocalData
    }
  })
}

/**
 * Returns the path to the user's picture directory.
 * # Linux
 * Resolves to `$XDG_PICTURES_DIR`.
 * # macOS
 * Resolves to `$HOME/Pictures`.
 * # Windows
 * Resolves to `{FOLDERID_Pictures}`.
 *
 * @returns
 */
async function pictureDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Picture
    }
  })
}

/**
 * Returns the path to the user's public directory.
 * # Linux
 * Resolves to `$XDG_PUBLICSHARE_DIR`.
 * # macOS
 * Resolves to `$HOME/Public`.
 * # Windows
 * Resolves to `{FOLDERID_Public}`.
 *
 * @returns
 */
async function publicDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Public
    }
  })
}

/**
 * Returns the path to the user's resource directory.
 *
 * @returns
 */
async function resourceDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Resource
    }
  })
}

/**
 * Returns the path to the user's runtime directory.
 * # Linux
 * Resolves to `$XDG_RUNTIME_DIR`.
 * # macOS
 * Not supported.
 * # Windows
 * Not supported.
 *
 * @returns
 */
async function runtimeDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Runtime
    }
  })
}

/**
 * Returns the path to the user's template directory.
 * # Linux
 * Resolves to `$XDG_TEMPLATES_DIR`.
 * # macOS
 * Not supported.
 * # Windows
 * Resolves to `{FOLDERID_Templates}`.
 *
 * @returns
 */
async function templateDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Template
    }
  })
}

/**
 * Returns the path to the user's video directory.
 * # Linux
 * Resolves to `$XDG_VIDEOS_DIR`.
 * # macOS
 * Resolves to `$HOME/Movies`.
 * # Windows
 * Resolves to `{FOLDERID_Videos}`.
 *
 * @returns
 */
async function videoDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Video
    }
  })
}

/**
 * Returns the path to the current working directory.
 *
 * @returns
 */
async function currentDir(): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path: '',
      directory: BaseDirectory.Current
    }
  })
}

/**
 * Resolves the path with the optional base directory.
 *
 * @param path A path to resolve
 * @param directory A base directory to use when resolving the given path
 * @returns A path resolved to the given base directory.
 */
async function resolvePath(
  path: string,
  directory: BaseDirectory
): Promise<string> {
  return invokeTauriCommand<string>({
    __tauriModule: 'Fs',
    message: {
      cmd: 'resolvePath',
      path,
      directory
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
  runtimeDir,
  templateDir,
  videoDir,
  currentDir,
  resolvePath
}
