// Copyright 2019-2021 Tauri Programme within The Commons Conservancy and Contributors
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

import { invoke } from '../tauri'

export type TauriModule =
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

export interface TauriCommand {
  __tauriModule: TauriModule
  mainThread?: boolean
  [key: string]: unknown
}

export async function invokeTauriCommand<T>(command: TauriCommand): Promise<T> {
  return invoke('tauri', command)
}
