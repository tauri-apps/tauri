// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */ /** */

import 'regenerator-runtime/runtime'
import * as app from './app'
import * as cli from './cli'
import * as dialog from './dialog'
import * as event from './event'
import * as fs from './fs'
import * as globalShortcut from './globalShortcut'
import * as http from './http'
import * as notification from './notification'
import * as path from './path'
import * as process from './process'
import * as shell from './shell'
import * as tauri from './tauri'
import * as updater from './updater'
import * as window from './window'
const invoke = tauri.invoke

export {
  app,
  cli,
  dialog,
  event,
  fs,
  globalShortcut,
  http,
  notification,
  path,
  process,
  shell,
  tauri,
  updater,
  window,
  invoke
}
