// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { BaseDirectory } from './fs'

/**
 * Returns the path to the suggested directory for your app config files
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's audio directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's cache directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's config directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's data directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's desktop directory

 * @returns {Promise<string>}
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
 * Returns the path to the user's document directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's download directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's executable directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's font directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's home directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's local data directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's picture directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's public directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's resource directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's runtime directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's template directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the user's video directory
 *
 * @returns {Promise<string>}
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
 * Returns the path to the current working directory
 *
 * @returns {Promise<string>}
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
 * Resolves the path with the optional base directory
 *
 * @param {string} path
 * @param {BaseDirectory} directory
 * @returns {Promise<string>}
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
