// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '../tauri'

/** @ignore */
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

/** @ignore */
export interface TauriCommand {
  __tauriModule: TauriModule
  mainThread?: boolean
  [key: string]: unknown
}

/** @ignore */
export async function invokeTauriCommand<T>(command: TauriCommand): Promise<T> {
  return invoke('tauri', command)
}
