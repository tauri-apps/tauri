// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invokeTauriCommand } from './helpers/tauri'
import { BaseDirectory } from './fs'

/**
 * @name appDir
 * @description Returns the path to the suggested directory for your app config files.
 * @return {Promise<string>}
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
 * @name audioDir
 * @description Returns the path to the user's audio directory.
 * @return {Promise<string>}
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
 * @name cacheDir
 * @description Returns the path to the user's cache directory.
 * @return {Promise<string>}
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
 * @name configDir
 * @description Returns the path to the user's config directory.
 * @return {Promise<string>}
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
 * @name dataDir
 * @description Returns the path to the user's data directory.
 * @return {Promise<string>}
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
 * @name desktopDir
 * @description Returns the path to the user's desktop directory.
 * @return {Promise<string>}
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
 * @name documentDir
 * @description Returns the path to the user's document directory.
 * @return {Promise<string>}
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
 * @name downloadDir
 * @description Returns the path to the user's download directory.
 * @return {Promise<string>}
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
 * @name executableDir
 * @description Returns the path to the user's executable directory.
 * @return {Promise<string>}
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
 * @name fontDir
 * @description Returns the path to the user's font directory.
 * @return {Promise<string>}
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
 * @name homeDir
 * @description Returns the path to the user's home directory.
 * @return {Promise<string>}
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
 * @name localDataDir
 * @description Returns the path to the user's local data directory.
 * @return {Promise<string>}
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
 * @name pictureDir
 * @description Returns the path to the user's picture directory.
 * @return {Promise<string>}
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
 * @name publicDir
 * @description Returns the path to the user's public directory.
 * @return {Promise<string>}
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
 * @name resourceDir
 * @description Returns the path to the user's resource directory.
 * @return {Promise<string>}
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
 * @name runtimeDir
 * @descriptionReturns Returns the path to the user's runtime directory.
 * @return {Promise<string>}
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
 * @name templateDir
 * @descriptionReturns Returns the path to the user's template directory.
 * @return {Promise<string>}
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
 * @name videoDir
 * @descriptionReturns Returns the path to the user's video dir.
 * @return {Promise<string>}
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
 * @name currentDir
 * @descriptionReturns Returns the path to the current working dir.
 * @return {Promise<string>}
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
 * @name resolvePath
 * @descriptionReturns Resolves the path with the optional base directory.
 * @return {Promise<string>}
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
