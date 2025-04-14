// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The Tauri API allows you to interface with the backend layer.
 *
 * This module exposes all other modules as an object where the key is the module name, and the value is the module exports.
 * @example
 * ```typescript
 * import { event, window, path } from '@tauri-apps/api'
 * ```
 *
 * ### Vanilla JS API
 *
 * The above import syntax is for JavaScript/TypeScript with a bundler. If you're using vanilla JavaScript, you can use the global `window.__TAURI__` object instead. It requires `app.withGlobalTauri` configuration option enabled.
 *
 * @example
 * ```js
 * const { event, window: tauriWindow, path } = window.__TAURI__;
 * ```
 *
 * @module
 */

import * as app from './app'
import * as core from './core'
import * as dpi from './dpi'
import * as event from './event'
import * as image from './image'
import * as menu from './menu'
import * as mocks from './mocks'
import * as path from './path'
import * as tray from './tray'
import * as webview from './webview'
import * as webviewWindow from './webviewWindow'
import * as window from './window'

export {
  app,
  core,
  dpi,
  event,
  image,
  menu,
  mocks,
  path,
  tray,
  webview,
  webviewWindow,
  window
}
