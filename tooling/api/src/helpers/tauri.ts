// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */

import { invoke } from '../tauri'

type TauriModule =
  | 'App'
  | 'Fs'
  | 'Path'
  | 'Os'
  | 'Window'
  | 'Shell'
  | 'Event'
  | 'Internal'
  | 'Dialog'
  | 'Cli'
  | 'Notification'
  | 'Http'
  | 'GlobalShortcut'
  | 'Process'
  | 'Clipboard'

interface TauriCommand {
  __tauriModule: TauriModule
  [key: string]: unknown
}

async function invokeTauriCommand<T>(command: TauriCommand): Promise<T> {
  return invoke('tauri', command)
}

export type { TauriModule, TauriCommand }

export { invokeTauriCommand }
