// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
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
 * @module
 */

import * as app from './app'
import * as event from './event'
import * as core from './core'
import * as window from './window'
import * as webview from './webview'
import * as path from './path'
import * as dpi from './dpi'
import * as tray from './tray'
import * as menu from './menu'

export { app, dpi, event, path, core, window, webview, tray, menu }
