// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/**
 * The Tauri API allows you to interface with the backend layer.
 *
 * This module exposes all other modules as an object where the key is the module name, and the value is the module exports.
 * @example
 * ```typescript
 * import { app, dialog, event, fs, globalShortcut } from '@tauri-apps/api'
 * ```
 * @module
 */

import * as event from './event'
import * as path from './path'
import * as process from './process'
import * as tauri from './tauri'
import * as updater from './updater'
import * as window from './window'

/** @ignore */
const invoke = tauri.invoke

export { invoke, event, path, process, tauri, updater, window }
