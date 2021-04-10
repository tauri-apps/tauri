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

export interface TauriCommand {
  __tauriModule: TauriModule
  mainThread?: boolean
  [key: string]: unknown
}

export async function invokeTauriCommand<T>(command: TauriCommand): Promise<T> {
  return invoke('tauri', command)
}
