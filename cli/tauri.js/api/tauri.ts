import { HttpOptions, EventCallback } from './models'

declare global {
  interface Window {
    tauri: {
      // window
      setTitle: (title: string) => void
      open: (url: string) => void

      // process
      execute: (command: string, args?: string | string[]) => Promise<string>

      // http
      httpRequest: <T>(options: HttpOptions) => Promise<T>

      // event
      listen: (event: string, handler: EventCallback) => void
      emit: (event: string, payload?: string) => void
    }
  }
}

export default window.tauri
