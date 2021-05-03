// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

/** @ignore */ /** */

import { invoke } from '../tauri'

export type TauriModule =
  | 'App'
  | 'Fs'
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

export interface TauriCommand {
  __tauriModule: TauriModule
  mainThread?: boolean
  [key: string]: unknown
}

export async function invokeTauriCommand<T>(command: TauriCommand): Promise<T> {
  return invoke('tauri', command)
}
