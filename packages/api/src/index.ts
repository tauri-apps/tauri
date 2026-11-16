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

import * as app from './app.js'
import * as core from './core.js'
import * as dpi from './dpi.js'
import * as event from './event.js'
import * as image from './image.js'
import * as menu from './menu.js'
import * as mocks from './mocks.js'
import * as path from './path.js'
import * as tray from './tray.js'
import * as webview from './webview.js'
import * as webviewWindow from './webviewWindow.js'
import * as window from './window.js'

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
