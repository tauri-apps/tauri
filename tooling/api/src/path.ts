// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { BaseDirectory } from './fs'

/**
 * Returns the path to the suggested directory for your app config files.
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
async function resolve(
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
  resolve as resolvePath
}
